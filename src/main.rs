mod git;
mod auth;
mod models;
mod db;
mod errors;
mod api;
mod repo;
mod frontend;

use crate::git::*;
use db::Database;
use std::env;
use dotenvy;

use actix_web::{web, App, HttpServer};
use git_http_backend::actix::handler::ActixGitHttp;
use git_http_backend::actix_git_router;
use git_http_backend::config::GitHttpConfig;
use std::io;
use std::path::PathBuf;
use std::fs;
use log::*;

#[tokio::main]
pub async fn main() -> io::Result<()> {
    tracing_subscriber::fmt().init();
    let _ = dotenvy::dotenv();

    let root = fs::canonicalize(PathBuf::from("./repos".to_string()))
        .unwrap_or_else(|_| PathBuf::from("./repos"));

    if !root.exists() {
        warn!("root path not exists");
        fs::create_dir(root.clone())?;
    }

    let db = Database::init().await;
    let db_data = web::Data::new(db);

    let addr =  String::from("localhost");
    let port: u16 = env::var("PORT").unwrap_or("8080".to_string()).parse().unwrap_or(8080);

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
            .app_data(web::Data::new(_auth.clone()))
            .wrap(actix_web::middleware::Logger::default())
            .configure(api::config)
            .service(
                utoipa_swagger_ui::SwaggerUi::new("/api/docs/{_:.*}")
                    .url("/api/docs/openapi.json", api::documentation::ApiDoc::openapi()),
            )
            .configure(frontend::config)
            .configure(|x| actix_git_router::<WithAuth>(x))
    })
        .bind(bind_addr)?
        .run()
        .await
}