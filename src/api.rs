use actix_web::{get, post, delete, web, HttpRequest, HttpResponse, Responder};
use mongodb::bson::{doc, oid::ObjectId, DateTime};
use futures_util::TryStreamExt;
use base64::Engine;

use crate::auth;
use crate::db::Database;
use crate::errors::{AuthError, GitError};
use crate::repo;
use crate::models::*;

fn bearer_token(req: &HttpRequest) -> Result<String, AuthError> {
    let header = req
        .headers()
        .get(actix_web::http::header::AUTHORIZATION)
        .ok_or(AuthError::MissingAuthHeader)?;
    let header = header.to_str().map_err(|_| AuthError::InvalidAuthHeader)?;
    let prefix = "Bearer ";
    if let Some(rest) = header.strip_prefix(prefix) {
        Ok(rest.to_string())
    } else {
        Err(AuthError::InvalidAuthHeader)
    }
}

async fn require_token_user_id(db: &Database, req: &HttpRequest) -> Result<ObjectId, AuthError> {
    let token = bearer_token(req)?;
    auth::auth(db, token).await
}

async fn repo_by_id(db: &Database, id_hex: &str) -> Result<Repository, HttpResponse> {
    let oid = ObjectId::parse_str(id_hex)
        .map_err(|_| HttpResponse::BadRequest().json(error_message("invalid id")))?;
    let filter = doc! { "_id": &oid };
    let repo = db
        .repositories
        .find_one(filter)
        .await
        .map_err(|e| HttpResponse::InternalServerError().json(error_message(&e.to_string())))?
        .ok_or_else(|| HttpResponse::NotFound().json(error_message("repository not found")))?;
    Ok(repo)
}

fn error_message(msg: &str) -> serde_json::Value {
    serde_json::json!({ "error": msg })
}

fn to_http_error<E: std::fmt::Display>(e: E) -> HttpResponse {
    HttpResponse::InternalServerError().json(error_message(&e.to_string()))
}

#[utoipa::path(
    post,
    path = "/api/v1/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Logged in successfully", body = LoginResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse)
    ),
    tag = "auth"
)]
#[post("/api/v1/login")]
async fn login(db: web::Data<Database>, payload: web::Json<LoginRequest>) -> impl Responder {
    match auth::login(&db, payload.login.clone(), payload.password.clone()).await {
        Ok(token) => HttpResponse::Ok().json(LoginResponse { token }),
        Err(AuthError::InvalidCredentials) => {
            HttpResponse::Unauthorized().json(error_message("invalid credentials"))
        }
        Err(e) => to_http_error(e),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/logout",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Logged out", body = OkResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "auth"
)]
#[post("/api/v1/logout")]
async fn logout(db: web::Data<Database>, req: HttpRequest) -> impl Responder {
    let token = match bearer_token(&req) {
        Ok(t) => t,
        Err(AuthError::MissingAuthHeader | AuthError::InvalidAuthHeader) => {
            return HttpResponse::Unauthorized().json(error_message("unauthorized"))
        }
        Err(e) => return to_http_error(e),
    };

    match auth::logout(&db, token).await {
        Ok(()) => HttpResponse::Ok().json(OkResponse { ok: true }),
        Err(AuthError::InvalidCredentials) => {
            HttpResponse::Unauthorized().json(error_message("unauthorized"))
        }
        Err(e) => to_http_error(e),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered", body = OkResponse),
        (status = 409, description = "Username or email already exists")
    ),
    tag = "auth"
)]
#[post("/api/v1/register")]
async fn register(db: web::Data<Database>, payload: web::Json<RegisterRequest>) -> impl Responder {
    match auth::register(
        &db,
        payload.username.clone(),
        payload.email.clone(),
        payload.password.clone(),
    )
        .await
    {
        Ok(()) => HttpResponse::Created().json(OkResponse { ok: true }),
        Err(AuthError::InvalidCredentials) => HttpResponse::Conflict().json(error_message(
            "username or email already exists",
        )),
        Err(e) => to_http_error(e),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/create",
    security(("bearerAuth" = [])),
    request_body = CreateRepoRequest,
    responses(
        (status = 201, description = "Repository created", body = Repository),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "Already exists")
    ),
    tag = "repos"
)]
#[post("/api/v1/create")]
async fn create_repo(
    db: web::Data<Database>,
    req: HttpRequest,
    payload: web::Json<CreateRepoRequest>,
) -> impl Responder {
    let user_id = match require_token_user_id(&db, &req).await {
        Ok(uid) => uid,
        Err(AuthError::MissingAuthHeader | AuthError::InvalidAuthHeader | AuthError::InvalidCredentials) => {
            return HttpResponse::Unauthorized().json(error_message("unauthorized"))
        }
        Err(e) => return to_http_error(e),
    };

    let name = payload.name.trim();
    if name.is_empty() {
        return HttpResponse::BadRequest().json(error_message("name must not be empty"));
    }

    match db
        .repositories
        .find_one(doc! { "user": &user_id, "name": name })
        .await
    {
        Ok(Some(_)) => {
            return HttpResponse::Conflict().json(error_message("already exists"));
        }
        Ok(None) => {}
        Err(e) => return to_http_error(e),
    }

    let now = DateTime::now();
    let repo_id = ObjectId::new();
    let repo_doc = Repository {
        _id: repo_id,
        user: user_id,
        name: name.to_string(),
        description: payload.description.clone().unwrap_or_default(),
        is_private: payload.is_private.unwrap_or(false),
        forked_from: None,
        created_at: now,
        updated_at: now,
    };

    if let Err(e) = db.create_repository(repo_doc.clone()).await {
        return to_http_error(e);
    }

    if let Err(e) = repo::init(user_id, repo_id).await {
        let _ = db.repositories.delete_one(doc! { "_id": &repo_id }).await;
        return to_http_error(e);
    }

    HttpResponse::Created().json(repo_doc)
}

#[utoipa::path(
    delete,
    path = "/api/v1/delete",
    security(("bearerAuth" = [])),
    params(DeleteQuery),
    responses(
        (status = 200, description = "Repository deleted", body = OkResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Repository not found")
    ),
    tag = "repos"
)]
#[delete("/api/v1/delete")]
async fn delete_repo(
    db: web::Data<Database>,
    req: HttpRequest,
    query: web::Query<DeleteQuery>,
) -> impl Responder {
    let uid = match require_token_user_id(&db, &req).await {
        Ok(u) => u,
        Err(AuthError::MissingAuthHeader | AuthError::InvalidAuthHeader | AuthError::InvalidCredentials) => {
            return HttpResponse::Unauthorized().json(error_message("unauthorized"))
        }
        Err(e) => return to_http_error(e),
    };

    let repository = match repo_by_id(&db, &query.id).await {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    if repository.user != uid {
        return HttpResponse::Forbidden().json(error_message("forbidden"));
    }

    let path = crate::repo::repo_path(&repository.user, &repository._id);
    if let Err(e) = tokio::fs::remove_dir_all(&path).await {
        if e.kind() != std::io::ErrorKind::NotFound {
            return to_http_error(e);
        }
    }

    match db.repositories.delete_one(doc! { "_id": &repository._id }).await {
        Ok(res) if res.deleted_count == 1 => HttpResponse::Ok().json(OkResponse { ok: true }),
        Ok(_) => HttpResponse::InternalServerError().json(error_message("failed to delete repository")),
        Err(e) => to_http_error(e),
    }
}


#[utoipa::path(
    get,
    path = "/api/v1/repos",
    params(ReposQuery),
    responses(
        (status = 200, description = "List repositories", body = [Repository])
    ),
    tag = "repos"
)]
#[get("/api/v1/repos")]
async fn list_repos(db: web::Data<Database>, req: HttpRequest, query: web::Query<ReposQuery>) -> impl Responder {
    let owner_user_id: Option<ObjectId> = if let Some(owner_username) = &query.owner {
        match db.find_user_by_login(owner_username).await {
            Ok(Some(u)) => Some(u._id),
            Ok(None) => return HttpResponse::Ok().json(Vec::<Repository>::new()),
            Err(e) => return to_http_error(e),
        }
    } else {
        None
    };

    let requester_user_id = match bearer_token(&req) {
        Ok(t) => match auth::auth(&db, t).await {
            Ok(uid) => Some(uid),
            Err(_) => None,
        },
        Err(_) => None,
    };

    let mut text_or = vec![];
    if let Some(q) = &query.q {
        if !q.trim().is_empty() {
            text_or.push(doc! { "name": { "$regex": q, "$options": "i" } });
            text_or.push(doc! { "description": { "$regex": q, "$options": "i" } });
        }
    }

    let privacy_filter = if let Some(owner_id) = owner_user_id {
        let can_see_private = requester_user_id.map_or(false, |r| r == owner_id);
        if can_see_private {
            doc! { "user": &owner_id }
        } else {
            doc! { "user": &owner_id, "is_private": false }
        }
    } else {
        if let Some(uid) = requester_user_id {
            doc! {
                "$or": [
                    { "is_private": false },
                    { "is_private": true, "user": &uid }
                ]
            }
        } else {
            doc! { "is_private": false }
        }
    };

    let mut filter = privacy_filter;
    if !text_or.is_empty() {
        filter.insert("$or", text_or);
    }

    let sort_doc = match query.filter.as_deref() {
        Some("newest") => doc! { "created_at": -1 },
        Some("updated") => doc! { "updated_at": -1 },
        _ => doc! { "updated_at": -1 }, // default last updated
    };

    let cursor = match db.repositories.find(filter).sort(sort_doc).await {
        Ok(c) => c,
        Err(e) => return to_http_error(e),
    };

    let repos: Vec<Repository> = match cursor.try_collect().await {
        Ok(v) => v,
        Err(e) => return to_http_error(e),
    };

    HttpResponse::Ok().json(repos)
}

#[utoipa::path(
    get,
    path = "/api/v1/branches",
    params(BranchesQuery),
    responses(
        (status = 200, description = "List branches", body = BranchesResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Repository not found")
    ),
    tag = "git"
)]
#[get("/api/v1/branches")]
async fn branches(db: web::Data<Database>, req: HttpRequest, query: web::Query<BranchesQuery>) -> impl Responder {
    let repo = match repo_by_id(&db, &query.id).await {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    // Access control
    if repo.is_private {
        let uid = match require_token_user_id(&db, &req).await {
            Ok(u) => u,
            Err(_) => return HttpResponse::Unauthorized().json(error_message("unauthorized")),
        };
        if uid != repo.user {
            return HttpResponse::Forbidden().json(error_message("forbidden"));
        }
    }

    match repo::list_branches(&repo.user, &repo._id).await {
        Ok(list) => HttpResponse::Ok().json(BranchesResponse { branches: list }),
        Err(e) => to_http_error(e),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/content",
    params(ContentQuery),
    responses(
        (status = 200, description = "Get tree or blob content", body = ContentResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Repository not found")
    ),
    tag = "git"
)]
#[get("/api/v1/content")]
async fn content(
    db: web::Data<Database>,
    req: HttpRequest,
    query: web::Query<ContentQuery>,
) -> impl Responder {
    let repo = match repo_by_id(&db, &query.id).await {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    if repo.is_private {
        let uid = match require_token_user_id(&db, &req).await {
            Ok(u) => u,
            Err(_) => return HttpResponse::Unauthorized().json(error_message("unauthorized")),
        };
        if uid != repo.user {
            return HttpResponse::Forbidden().json(error_message("forbidden"));
        }
    }

    let branch_opt = query.branch.as_deref();
    let rev = query
        .commit
        .as_deref()
        .unwrap_or_else(|| branch_opt.unwrap_or("HEAD"));
    let branch_for_lookup = if query.commit.is_some() { None } else { branch_opt };

    if let Some(path) = &query.path {
        match repo::get_file_content(&repo.user, &repo._id, rev, branch_for_lookup, path).await {
            Ok(bytes) => {
                let content_base64 = base64::engine::general_purpose::STANDARD.encode(bytes);
                return HttpResponse::Ok().json(ContentResponse::Blob { content_base64 });
            }
            Err(_e_blob) => {
                match repo::list_tree(&repo.user, &repo._id, rev, branch_for_lookup, Some(path)).await {
                    Ok(entries) => return HttpResponse::Ok().json(ContentResponse::Tree { entries }),
                    Err(e) => return to_http_error(e),
                }
            }
        }
    } else {
        match repo::list_tree(&repo.user, &repo._id, rev, branch_for_lookup, None).await {
            Ok(entries) => HttpResponse::Ok().json(ContentResponse::Tree { entries }),
            Err(e) => to_http_error(e),
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/commits",
    params(CommitsQuery),
    responses(
        (status = 200, description = "List commits", body = [CommitInfo]),
        (status = 400, description = "Invalid branch or revspec"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Repository not found")
    ),
    tag = "git"
)]
#[get("/api/v1/commits")]
async fn commits(
    db: web::Data<Database>,
    req: HttpRequest,
    query: web::Query<CommitsQuery>,
) -> impl Responder {
    let repo = match repo_by_id(&db, &query.id).await {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    if repo.is_private {
        let uid = match require_token_user_id(&db, &req).await {
            Ok(u) => u,
            Err(_) => return HttpResponse::Unauthorized().json(error_message("unauthorized")),
        };
        if uid != repo.user {
            return HttpResponse::Forbidden().json(error_message("forbidden"));
        }
    }

    let branch = query.branch.as_deref().unwrap_or("HEAD");
    let limit = query.limit.unwrap_or(50);

    match repo::list_commits(&repo.user, &repo._id, branch, Some(branch), limit).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(GitError::Git(msg)) if msg.contains("revspec") => {
            HttpResponse::BadRequest().json(error_message("invalid branch or revspec"))
        }
        Err(e) => to_http_error(e),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(login)
        .service(logout)
        .service(register)
        .service(create_repo)
        .service(delete_repo)
        .service(list_repos)
        .service(branches)
        .service(content)
        .service(commits);
}