use actix_web::{get, post, delete, web, HttpRequest, HttpResponse, Responder};

use crate::db::Database;
use crate::errors::AuthError;
use crate::api::service;
use crate::models::*;
use bson::oid::ObjectId;
use crate::db::Repository;

// ----------------- helpers -----------------

fn error_message(msg: &str) -> serde_json::Value {
    serde_json::json!({ "error": msg })
}

fn to_http_500<E: std::fmt::Display>(e: E) -> HttpResponse {
    HttpResponse::InternalServerError().json(error_message(&e.to_string()))
}

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

async fn optional_requester(db: &Database, req: &HttpRequest) -> Option<ObjectId> {
    match bearer_token(req) {
        Ok(token) => service::get_user_id_from_token(db, token)
            .await
            .ok(),
        Err(_) => None,
    }
}

// ----------------- auth -----------------

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
pub async fn login(db: web::Data<Database>, payload: web::Json<LoginRequest>) -> impl Responder {
    match service::auth_login(&db, payload.login.clone(), payload.password.clone()).await {
        Ok(token) => HttpResponse::Ok().json(LoginResponse { token }),
        Err(AuthError::InvalidCredentials) => {
            HttpResponse::Unauthorized().json(error_message("invalid credentials"))
        }
        Err(e) => to_http_500(e),
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
pub async fn logout(db: web::Data<Database>, req: HttpRequest) -> impl Responder {
    let token = match bearer_token(&req) {
        Ok(t) => t,
        Err(AuthError::MissingAuthHeader | AuthError::InvalidAuthHeader) => {
            return HttpResponse::Unauthorized().json(error_message("unauthorized"))
        }
        Err(e) => return to_http_500(e),
    };

    match service::auth_logout(&db, token).await {
        Ok(()) => HttpResponse::Ok().json(OkResponse { ok: true }),
        Err(AuthError::InvalidCredentials) => {
            HttpResponse::Unauthorized().json(error_message("unauthorized"))
        }
        Err(e) => to_http_500(e),
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
pub async fn register(db: web::Data<Database>, payload: web::Json<RegisterRequest>) -> impl Responder {
    match service::auth_register(
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
        Err(e) => to_http_500(e),
    }
}

// ----------------- repos -----------------

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
pub async fn create_repo(
    db: web::Data<Database>,
    req: HttpRequest,
    payload: web::Json<CreateRepoRequest>,
) -> impl Responder {
    let user_id = match bearer_token(&req) {
        Ok(t) => match service::get_user_id_from_token(&db, t).await {
            Ok(uid) => uid,
            Err(AuthError::InvalidCredentials) => {
                return HttpResponse::Unauthorized().json(error_message("unauthorized"))
            }
            Err(e) => return to_http_500(e),
        },
        Err(AuthError::MissingAuthHeader | AuthError::InvalidAuthHeader) => {
            return HttpResponse::Unauthorized().json(error_message("unauthorized"))
        }
        Err(e) => return to_http_500(e),
    };

    match service::repo_create(&db, user_id, payload.into_inner()).await {
        Ok(repo) => HttpResponse::Created().json(repo),
        Err(msg) if msg == "name must not be empty" => HttpResponse::BadRequest().json(error_message(&msg)),
        Err(msg) if msg == "already exists" => HttpResponse::Conflict().json(error_message(&msg)),
        Err(e) => to_http_500(e),
    }
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
pub async fn delete_repo(
    db: web::Data<Database>,
    req: HttpRequest,
    query: web::Query<DeleteQuery>,
) -> impl Responder {
    let requester = match bearer_token(&req) {
        Ok(t) => match service::get_user_id_from_token(&db, t).await {
            Ok(uid) => uid,
            Err(AuthError::InvalidCredentials) => {
                return HttpResponse::Unauthorized().json(error_message("unauthorized"))
            }
            Err(e) => return to_http_500(e),
        },
        Err(AuthError::MissingAuthHeader | AuthError::InvalidAuthHeader) => {
            return HttpResponse::Unauthorized().json(error_message("unauthorized"))
        }
        Err(e) => return to_http_500(e),
    };

    match service::repo_delete(&db, requester, &query.id).await {
        Ok(()) => HttpResponse::Ok().json(OkResponse { ok: true }),
        Err(msg) if msg == "forbidden" => HttpResponse::Forbidden().json(error_message(&msg)),
        Err(msg) if msg == "repository not found" => HttpResponse::NotFound().json(error_message(&msg)),
        Err(e) => to_http_500(e),
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
pub async fn list_repos(db: web::Data<Database>, req: HttpRequest, query: web::Query<ReposQuery>) -> impl Responder {
    let requester = optional_requester(&db, &req).await;
    match service::repo_list(&db, requester, query.into_inner()).await {
        Ok(repos) => HttpResponse::Ok().json(repos),
        Err(e) => to_http_500(e),
    }
}

// ----------------- git browsing -----------------

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
pub async fn branches(db: web::Data<Database>, req: HttpRequest, query: web::Query<BranchesQuery>) -> impl Responder {
    let requester = optional_requester(&db, &req).await;
    match service::git_branches(&db, requester.clone(), &query.id).await {
        Ok(list) => HttpResponse::Ok().json(BranchesResponse { branches: list }),
        Err(msg) if msg == "forbidden" => {
            if requester.is_none() {
                HttpResponse::Unauthorized().json(error_message("unauthorized"))
            } else {
                HttpResponse::Forbidden().json(error_message("forbidden"))
            }
        }
        Err(msg) if msg == "repository not found" => HttpResponse::NotFound().json(error_message(&msg)),
        Err(e) => to_http_500(e),
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/branch",
    params(DeleteBranchQuery),
    responses(
        (status = 200, description = "Branch deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Repository not found"),
        (status = 400, description = "Bad request")
    ),
    tag = "git"
)]
#[delete("/api/v1/branch")]
pub async fn delete_branch(
    db: web::Data<Database>,
    req: HttpRequest,
    query: web::Query<DeleteBranchQuery>,
) -> impl Responder {
    let requester = optional_requester(&db, &req).await;

    match service::git_remove_branch(&db, requester.clone(), &query.id, &query.branch).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Branch '{}' deleted", query.branch)
        })),
        Err(msg) if msg == "forbidden" => {
            if requester.is_none() {
                HttpResponse::Unauthorized().json(error_message("unauthorized"))
            } else {
                HttpResponse::Forbidden().json(error_message("forbidden"))
            }
        }
        Err(msg) if msg == "repository not found" => {
            HttpResponse::NotFound().json(error_message(&msg))
        }
        Err(msg) if msg.contains("cannot delete branch") => {
            HttpResponse::BadRequest().json(error_message(&msg))
        }
        Err(e) => to_http_500(e),
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
pub async fn content(
    db: web::Data<Database>,
    req: HttpRequest,
    query: web::Query<ContentQuery>,
) -> impl Responder {
    let requester = optional_requester(&db, &req).await;
    match service::git_content(&db, requester.clone(), query.into_inner()).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(msg) if msg == "forbidden" => {
            if requester.is_none() {
                HttpResponse::Unauthorized().json(error_message("unauthorized"))
            } else {
                HttpResponse::Forbidden().json(error_message("forbidden"))
            }
        }
        Err(msg) if msg == "repository not found" => HttpResponse::NotFound().json(error_message(&msg)),
        Err(e) => to_http_500(e),
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
pub async fn commits(
    db: web::Data<Database>,
    req: HttpRequest,
    query: web::Query<CommitsQuery>,
) -> impl Responder {
    let requester = optional_requester(&db, &req).await;
    match service::git_commits(&db, requester.clone(), query.into_inner()).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(msg) if msg == "forbidden" => {
            if requester.is_none() {
                HttpResponse::Unauthorized().json(error_message("unauthorized"))
            } else {
                HttpResponse::Forbidden().json(error_message("forbidden"))
            }
        }
        Err(msg) if msg == "invalid branch or revspec" => {
            HttpResponse::BadRequest().json(error_message("invalid branch or revspec"))
        }
        Err(msg) if msg == "repository not found" => HttpResponse::NotFound().json(error_message(&msg)),
        Err(e) => to_http_500(e),
    }
}

// Download as ZIP
#[utoipa::path(
    get,
    path = "/api/v1/download",
    params(ContentQuery),
    responses(
        (status = 200, description = "ZIP archive of the specified path", content_type = "application/zip"),
        (status = 400, description = "Invalid branch or revspec"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Repository not found")
    ),
    tag = "git"
)]
#[get("/api/v1/download")]
pub async fn download(
    db: web::Data<Database>,
    req: HttpRequest,
    query: web::Query<ContentQuery>,
) -> impl Responder {
    let requester = optional_requester(&db, &req).await;
    match service::git_download(&db, requester.clone(), query.into_inner()).await {
        Ok((filename, bytes)) => {
            use actix_web::http::header::{ContentDisposition, DispositionType, DispositionParam};
            let cd = ContentDisposition {
                disposition: DispositionType::Attachment,
                parameters: vec![DispositionParam::Filename(filename)],
            };
            HttpResponse::Ok()
                .content_type("application/zip")
                .insert_header(cd)
                .body(bytes)
        }
        Err(msg) if msg == "forbidden" => {
            if requester.is_none() {
                HttpResponse::Unauthorized().json(error_message("unauthorized"))
            } else {
                HttpResponse::Forbidden().json(error_message("forbidden"))
            }
        }
        Err(msg) if msg == "invalid branch or revspec" => {
            HttpResponse::BadRequest().json(error_message("invalid branch or revspec"))
        }
        Err(msg) if msg == "repository not found" => HttpResponse::NotFound().json(error_message(&msg)),
        Err(e) => to_http_500(e),
    }
}

// ----------------- actix config -----------------

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(login)
        .service(logout)
        .service(register)
        .service(create_repo)
        .service(delete_repo)
        .service(list_repos)
        .service(branches)
        .service(delete_branch)       
        .service(content)
        .service(commits)
        .service(download);
}