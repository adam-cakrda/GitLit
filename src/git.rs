use clap::Parser;
use git_http_backend::actix::handler::ActixGitHttp;
use git_http_backend::{AuthInput, GitConfig, GitOperation};
use std::path::PathBuf;
use tracing;
use std::fs;
use http_auth_basic::Credentials;

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
        let expected = Credentials::new("username", "password");
        if let Some(h) = auth.authorization {
            let credentials = Credentials::from_header(h).unwrap();
            tracing::info!(credentials.user_id, credentials.password);
            if credentials == expected {
                return Ok(());
            }
        }
        Err(())
    }

    async fn is_public_repo(&self, repo_path: &std::path::Path) -> bool {
        true
    }

     async fn allow_anonymous(&self, op: GitOperation) -> bool {
        self.inner.allow_anonymous(op).await
    }
}
