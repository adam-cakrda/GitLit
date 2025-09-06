use clap::Parser;
use git_http_backend::{AuthInput, GitConfig, GitOperation, actix::handler::ActixGitHttp};
use std::path::PathBuf;
use tracing;
use std::fs;
use http_auth_basic::Credentials;
use crate::db::Database;
use crate::repo::repo_path;
use mongodb::bson::doc;
use bcrypt::verify;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct ActixServerArgs {
    #[arg(short, long, default_value = "e:")]
    pub root: String,
    #[arg(short, long, default_value = "80")]
    pub port: u16,
    #[arg(short, long, default_value = "0.0.0.0")]
    pub addr: String,
}

#[derive(Clone, Debug)]
pub struct WithAuth {
    pub inner: ActixGitHttp,
    pub db: Database,
}

#[async_trait::async_trait]
impl GitConfig for WithAuth {
    async fn rewrite(&self, original_path: String) -> PathBuf {
        let trimmed = original_path.trim_start_matches('/');
        let segments: Vec<&str> = trimmed.split('/').collect();
        tracing::info!("rewrite: original path: {:?}", original_path);

        if segments.len() < 2 {
            tracing::warn!("rewrite: unexpected path '{}'", original_path);
            let fallback = PathBuf::from("./repos".to_string() + &original_path);
            return fs::canonicalize(&fallback).unwrap_or(fallback);
        }

        let username = segments[0];
        let mut reponame = segments[1];
        if let Some(no_git) = reponame.strip_suffix(".git") {
            reponame = no_git;
        }
        let rest = if segments.len() > 2 {
            Some(segments[2..].join("/"))
        } else {
            None
        };

        let resolved_path = if let Ok(Some(user)) = self.db.find_user_by_login(username).await {
            match self.db.repositories.find_one(doc! { "user": &user._id, "name": reponame }).await {
                Ok(Some(repo)) => {
                    let base = repo_path(&user._id, &repo._id);
                    if let Some(r) = rest.as_deref() {
                        base.join(r)
                    } else {
                        base
                    }
                }
                _ => PathBuf::from("./repos".to_string() + &original_path),
            }
        } else {
            PathBuf::from("./repos".to_string() + &original_path)
        };

        tracing::info!("rewrite: resolved path: {:?}", resolved_path);

        fs::canonicalize(&resolved_path).unwrap_or(resolved_path)
    }

    async fn authenticate(&self, auth: AuthInput) -> Result<(), ()> {
        if let Some(h) = auth.authorization {
            if let Ok(credentials) = Credentials::from_header(h.clone()) {
                let login = credentials.user_id;
                let password = credentials.password;
                tracing::info!("Authenticating with Basic credentials (username/password), login={}", login);

                match self.db.find_user_by_login(&login).await {
                    Ok(Some(user)) => {
                        match verify(password, &user.password) {
                            Ok(true) => {
                                tracing::debug!("Authentication successful for user '{}'", login);
                                Ok(())
                            }
                            Ok(false) => {
                                tracing::warn!("Authentication failed: invalid password for user '{}'", login);
                                Err(())
                            }
                            Err(e) => {
                                tracing::warn!("Authentication error during password verify for '{}': {}", login, e);
                                Err(())
                            }
                        }
                    }
                    Ok(None) => {
                        tracing::warn!("Authentication failed: user '{}' not found", login);
                        Err(())
                    }
                    Err(e) => {
                        tracing::warn!("Authentication error looking up user '{}': {}", login, e);
                        Err(())
                    }
                }
            } else {
                tracing::warn!("Unsupported Authorization header format");
                Err(())
            }
        } else {
            tracing::warn!("Missing Authorization header");
            Err(())
        }
    }

    async fn is_public_repo(&self, repo_path: &std::path::Path) -> bool {
        tracing::debug!("Checking if repo is public: {:?}", repo_path);
        let components: Vec<String> = repo_path
            .components()
            .filter_map(|c| c.as_os_str().to_str().map(|s| s.to_string()))
            .collect();

        let (user_hex, repo_hex) = match components.iter().position(|c| c == "repos") {
            Some(idx) if idx + 2 < components.len() => {
                (components[idx + 1].clone(), components[idx + 2].clone())
            }
            _ => {
                tracing::warn!("is_public_repo: could not locate repos/<user>/<repo> in path: {:?}", repo_path);
                return false;
            }
        };

        let user_oid = match mongodb::bson::oid::ObjectId::parse_str(&user_hex) {
            Ok(oid) => oid,
            Err(_) => {
                tracing::warn!("is_public_repo: invalid user ObjectId '{}'", user_hex);
                return false;
            }
        };
        let repo_oid = match mongodb::bson::oid::ObjectId::parse_str(&repo_hex) {
            Ok(oid) => oid,
            Err(_) => {
                tracing::warn!("is_public_repo: invalid repo ObjectId '{}'", repo_hex);
                return false;
            }
        };

        match self
            .db
            .repositories
            .find_one(mongodb::bson::doc! { "_id": &repo_oid, "user": &user_oid })
            .await
        {
            Ok(Some(repo)) => {
                let public = !repo.is_private;
                tracing::debug!(
                    "is_public_repo: repo {} (user {}) is {}",
                    repo_hex,
                    user_hex,
                    if public { "public" } else { "private" }
                );
                public
            }
            Ok(None) => {
                tracing::warn!(
                    "is_public_repo: repo not found for user={}, repo={}",
                    user_hex,
                    repo_hex
                );
                false
            }
            Err(e) => {
                tracing::warn!("is_public_repo: DB error: {}", e);
                false
            }
        }
    }

    async fn allow_anonymous(&self, op: GitOperation) -> bool {
        self.inner.allow_anonymous(op).await
    }
}
