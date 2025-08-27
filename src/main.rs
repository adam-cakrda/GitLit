mod git;
mod auth;
mod models;
mod db;
mod errors;
mod api;
mod repo;

use crate::git::*;
use crate::db::Database;

use actix_web::{web, App, HttpServer};
use git_http_backend::actix::handler::ActixGitHttp;
use git_http_backend::actix_git_router;
use git_http_backend::config::GitHttpConfig;
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

    let db = Database::init().await;
    let db_data = web::Data::new(db);

    let addr =  String::from("localhost");
    let port = 8080;

    let base = ActixGitHttp {
        config: GitHttpConfig {
            root,
            port,
            addr: addr.clone(),
        },
    };

    let _auth = WithAuth { inner: base, db: db_data.get_ref().clone() };

    let bind_addr = format!("{}:{}", addr.clone(), port);
    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .wrap(actix_web::middleware::Logger::default())
            .configure(|x| actix_git_router::<WithAuth>(x))
            .configure(crate::api::config)
    })
        .bind(bind_addr)?
        .run()
        .await
}