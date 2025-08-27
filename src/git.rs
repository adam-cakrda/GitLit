use clap::Parser;
use git_http_backend::actix::handler::ActixGitHttp;
use git_http_backend::{AuthInput, GitConfig, GitOperation};
use std::path::PathBuf;
use tracing;
use std::fs;
use http_auth_basic::Credentials;
use crate::db::Database;
use crate::auth::auth as auth_check;

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
        let path =
            fs::canonicalize(
                PathBuf::from(
                    "./repos".to_string()
                        + &original_path
                )
            ).unwrap();

        path
    }

    async fn authenticate(&self, auth: AuthInput) -> Result<(), ()> {
        if let Some(h) = auth.authorization {
            if let Ok(credentials) = Credentials::from_header(h.clone()) {
                let token = credentials.password;
                tracing::debug!("Authenticating with Basic credentials (password as token), user_id={}", credentials.user_id);
                return match auth_check(&self.db, token).await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        tracing::warn!("Basic auth failed: {}", e);
                        Err(())
                    }
                };
            }
            tracing::warn!("Unsupported Authorization header format");
        } else {
            tracing::warn!("Missing Authorization header");
        }
        Err(())
    }

    async fn is_public_repo(&self, repo_path: &std::path::Path) -> bool {
        // TODO: check if repo is public
        true
    }

    async fn allow_anonymous(&self, op: GitOperation) -> bool {
        self.inner.allow_anonymous(op).await
    }
}