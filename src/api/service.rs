use mongodb::bson::{oid::ObjectId};
use crate::db::Database;
use crate::errors::*;
use crate::models::*;
use mongodb::bson::doc;
use tokio::fs;
use std::io::Write;

// AUTH
pub async fn auth_register(db: &Database, username: String, email: String, password: String) -> Result<(), AuthError> {
    crate::auth::register(db, username, email, password).await
}

pub async fn auth_login(db: &Database, login: String, password: String) -> Result<String, AuthError> {
    crate::auth::login(db, login, password).await
}

pub async fn auth_logout(db: &Database, token: String) -> Result<(), AuthError> {
    crate::auth::logout(db, token).await
}

// HELPERS
pub async fn resolve_repo_by_id(db: &Database, id_hex: &str) -> Result<Repository, String> {
    use mongodb::bson::oid::ObjectId;
    let oid = ObjectId::parse_str(id_hex).map_err(|_| "invalid id".to_string())?;
    db.repositories
        .find_one(doc! { "_id": &oid })
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "repository not found".to_string())
}

pub async fn get_user_id_from_token(db: &Database, token: String) -> Result<ObjectId, AuthError> {
    crate::auth::auth(db, token).await
}

pub async fn username_by_id(db: &Database, user_id: &ObjectId) -> Result<Option<String>, String> {
    use mongodb::bson::doc;
    match db.users.find_one(doc! { "_id": user_id }).await {
        Ok(Some(user)) => Ok(Some(user.username)),
        Ok(None) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

// REPOS
pub async fn repo_create(db: &Database, user_id: ObjectId, payload: CreateRepoRequest) -> Result<Repository, String> {
    use mongodb::bson::{doc, DateTime};
    use mongodb::bson::oid::ObjectId;

    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err("name must not be empty".into());
    }

    match db.repositories.find_one(doc! { "user": &user_id, "name": &name }).await {
        Ok(Some(_)) => return Err("already exists".into()),
        Ok(None) => {},
        Err(e) => return Err(e.to_string()),
    }

    let now = DateTime::now();
    let repo_id = ObjectId::new();
    let repo_doc = Repository {
        _id: repo_id,
        user: user_id,
        name,
        description: payload.description.unwrap_or_default(),
        is_private: payload.is_private.unwrap_or(false),
        forked_from: None,
        created_at: now,
        updated_at: now,
    };

    db.create_repository(repo_doc.clone()).await.map_err(|e| e.to_string())?;
    if let Err(e) = crate::repo::init(repo_doc.user, repo_doc._id).await {
        let _ = db.repositories.delete_one(doc! { "_id": &repo_doc._id }).await;
        return Err(e.to_string());
    }

    Ok(repo_doc)
}

pub async fn repo_delete(db: &Database, requester: ObjectId, repo_id_hex: &str) -> Result<(), String> {
    let repository = resolve_repo_by_id(db, repo_id_hex).await?;
    if repository.user != requester {
        return Err("forbidden".into());
    }

    let path = crate::repo::repo_path(&repository.user, &repository._id);
    if let Err(e) = fs::remove_dir_all(&path).await {
        if e.kind() != std::io::ErrorKind::NotFound {
            return Err(e.to_string());
        }
    }

    match db.repositories.delete_one(doc! { "_id": &repository._id }).await {
        Ok(res) if res.deleted_count == 1 => Ok(()),
        Ok(_) => Err("failed to delete repository".into()),
        Err(e) => Err(e.to_string()),
    }
}

pub async fn repo_list(
    db: &Database,
    requester_user_id: Option<ObjectId>,
    query: ReposQuery,
) -> Result<Vec<Repository>, String> {
    use futures_util::TryStreamExt;
    use mongodb::bson::doc;

    let owner_user_id: Option<ObjectId> = if let Some(owner_username) = &query.owner {
        match db.find_user_by_login(owner_username).await {
            Ok(Some(u)) => Some(u._id),
            Ok(None) => return Ok(Vec::new()),
            Err(e) => return Err(e.to_string()),
        }
    } else {
        None
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
    } else if let Some(uid) = requester_user_id {
        doc! {
            "$or": [
                { "is_private": false },
                { "is_private": true, "user": &uid }
            ]
        }
    } else {
        doc! { "is_private": false }
    };

    let mut filter = privacy_filter;
    if !text_or.is_empty() {
        filter.insert("$or", text_or);
    }

    let sort_doc = match query.filter.as_deref() {
        Some("newest") => doc! { "created_at": -1 },
        Some("updated") | _ => doc! { "updated_at": -1 },
    };

    let cursor = db.repositories.find(filter).sort(sort_doc).await.map_err(|e| e.to_string())?;
    let repos: Vec<Repository> = cursor.try_collect().await.map_err(|e| e.to_string())?;
    Ok(repos)
}

// GIT
pub async fn git_branches(
    db: &Database,
    requester_user_id: Option<ObjectId>,
    repo_id_hex: &str,
) -> Result<Vec<Branch>, String> {
    let repo = resolve_repo_by_id(db, repo_id_hex).await?;
    if repo.is_private && Some(repo.user) != requester_user_id {
        return Err("forbidden".into());
    }
    crate::repo::list_branches(&repo.user, &repo._id).await.map_err(|e| e.to_string())
}

pub async fn git_content(
    db: &Database,
    requester_user_id: Option<ObjectId>,
    query: ContentQuery,
) -> Result<ContentResponse, String> {
    use base64::Engine;

    let repo = resolve_repo_by_id(db, &query.id).await?;
    if repo.is_private && Some(repo.user) != requester_user_id {
        return Err("forbidden".into());
    }

    let branch_opt = query.branch.as_deref();
    let rev = query.commit.as_deref().unwrap_or_else(|| branch_opt.unwrap_or("HEAD"));
    let branch_for_lookup = if query.commit.is_some() { None } else { branch_opt };

    if let Some(path) = &query.path {
        match crate::repo::get_file_content(&repo.user, &repo._id, rev, branch_for_lookup, path).await {
            Ok(bytes) => {
                let content_base64 = base64::engine::general_purpose::STANDARD.encode(bytes);
                Ok(ContentResponse::Blob { content_base64 })
            }
            Err(_) => {
                let entries = crate::repo::list_tree(&repo.user, &repo._id, rev, branch_for_lookup, Some(path))
                    .await.map_err(|e| e.to_string())?;
                Ok(ContentResponse::Tree { entries })
            }
        }
    } else {
        let entries = crate::repo::list_tree(&repo.user, &repo._id, rev, branch_for_lookup, None)
            .await.map_err(|e| e.to_string())?;
        Ok(ContentResponse::Tree { entries })
    }
}

pub async fn git_commits(
    db: &Database,
    requester_user_id: Option<ObjectId>,
    query: CommitsQuery,
) -> Result<Vec<CommitInfo>, String> {
    let repo = resolve_repo_by_id(db, &query.id).await?;
    if repo.is_private && Some(repo.user) != requester_user_id {
        return Err("forbidden".into());
    }

    let branch = query.branch.as_deref().unwrap_or("HEAD");
    let limit = query.limit.unwrap_or(50);

    crate::repo::list_commits(&repo.user, &repo._id, branch, Some(branch), limit)
        .await
        .map_err(|e| e.to_string())
}

pub async fn git_download(
    db: &Database,
    requester_user_id: Option<ObjectId>,
    query: ContentQuery,
) -> Result<(String, Vec<u8>), String> {
    use zip::write::FileOptions;
    use zip::CompressionMethod;

    let repo = resolve_repo_by_id(db, &query.id).await?;
    if repo.is_private && Some(repo.user) != requester_user_id {
        return Err("forbidden".into());
    }

    let branch_opt = query.branch.as_deref();
    let rev = query
        .commit
        .as_deref()
        .unwrap_or_else(|| branch_opt.unwrap_or("HEAD"));
    let branch_for_lookup = if query.commit.is_some() { None } else { branch_opt };

    let files = crate::repo::collect_files_at_path(&repo.user, &repo._id, rev, branch_for_lookup, query.path.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    if files.is_empty() {
        return Err("invalid branch or revspec".into());
    }

    let default_name = query
        .path
        .as_deref()
        .and_then(|p| std::path::Path::new(p).file_name().and_then(|s| s.to_str()))
        .map(|s| format!("{}.zip", s))
        .unwrap_or_else(|| format!("{}-{}.zip", repo.name, branch_opt.unwrap_or("HEAD")));

    let mut buf: Vec<u8> = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));

        let options = FileOptions::<()>::default()
            .compression_method(CompressionMethod::Deflated);

        for (path, bytes) in files {
            zip.start_file(path, options.clone())
                .map_err(|e| e.to_string())?;
            zip.write_all(&bytes).map_err(|e| e.to_string())?;
        }

        zip.finish().map_err(|e| e.to_string())?;
    }

    Ok((default_name, buf))
}