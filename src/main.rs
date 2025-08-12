mod git;
mod auth;

use crate::git::*;

use actix_web::{web, App, HttpServer};
use git_http_backend::actix::handler::ActixGitHttp;
use git_http_backend::actix_git_router;
use git_http_backend::config::GitHttpConfig;
use git_http_backend::{GitConfig, GitOperation};
use std::io;
use std::path::PathBuf;
use std::fs;
use log::warn;

#[tokio::main]
pub async fn main() -> io::Result<()> {
    tracing_subscriber::fmt().init();

    let root = fs::canonicalize(PathBuf::from("./repos".to_string()))?;

    if !root.exists() {
        warn!("root path not exists");
        fs::create_dir_all(root.clone())?;
    }

    let addr =  String::from("localhost");
    let port = 8080;

    let base = ActixGitHttp {
        config: GitHttpConfig {
            root,
            port,
            addr: addr.clone(),
        },
    };

    let auth = WithAuth { inner: base };

    let bind_addr = format!("{}:{}", addr.clone(), port);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(auth.clone()))
            .wrap(actix_web::middleware::Logger::default())
            .configure(|x| actix_git_router::<WithAuth>(x))
    })
        .bind(bind_addr)?
        .run()
        .await
}