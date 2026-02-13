mod api;
mod auth;
mod db;
mod errors;
mod frontend;
mod git;
mod models;
mod repo;

use crate::git::*;
use db::Database;
use dotenvy;
use std::env;

use actix_web::{App, HttpServer, web};
use git_http_backend::actix::handler::ActixGitHttp;
use git_http_backend::actix_git_router;
use git_http_backend::config::GitHttpConfig;
use std::{fs, io, path::Path};

#[tokio::main]
pub async fn main() -> io::Result<()> {
    tracing_subscriber::fmt().init();
    let _ = dotenvy::dotenv();

    let p = Path::new("repos");
    fs::create_dir_all(p)?;
    let root = fs::canonicalize(p)?;

    let db = Database::init().await;
    let db_data = web::Data::new(db);

    let addr = String::from("localhost");
    let port: u16 = env::var("PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .unwrap_or(8080);

    let api_config = env::var("API").unwrap_or("/api".to_string());
    let (_api_port, _api_prefix): (Option<u16>, String) =
        if let Ok(port_num) = api_config.parse::<u16>() {
            (Some(port_num), String::new())
        } else {
            (None, api_config)
        };


    let base = ActixGitHttp {
        config: GitHttpConfig {
            root,
            port,
            addr: addr.clone(),
        },
    };

    let _auth = WithAuth {
        inner: base,
        db: db_data.get_ref().clone(),
    };

    let bind_addr = format!("{}:{}", addr.clone(), port);
    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .app_data(web::Data::new(_auth.clone()))
            .wrap(actix_web::middleware::Logger::default())
            .configure(api::config)
            .service(utoipa_swagger_ui::SwaggerUi::new("/api/docs/{_:.*}").url(
                "/api/docs/openapi.json",
                api::documentation::ApiDoc::openapi(),
            ))
            .configure(frontend::config)
            .configure(|x| actix_git_router::<WithAuth>(x))
    })
    .bind(bind_addr)?
    .run()
    .await
}
