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
use utoipa::OpenApi;

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        crate::api::login,
        crate::api::logout,
        crate::api::register,
        crate::api::create_repo,
        crate::api::delete_repo,
        crate::api::list_repos,
        crate::api::branches,
        crate::api::content,
        crate::api::commits,
        crate::api::download
    ),
    components(
        schemas(
            // auth/api
            crate::models::LoginRequest,
            crate::models::LoginResponse,
            crate::models::RegisterRequest,
            crate::models::CreateRepoRequest,
            crate::models::DeleteQuery,
            crate::models::OkResponse,
            crate::models::ReposQuery,
            crate::models::BranchesQuery,
            crate::models::BranchesResponse,
            crate::models::ContentQuery,
            crate::models::ContentResponse,
            crate::models::CommitsQuery,
            // db models
            crate::models::Repository,
            crate::models::Token,
            crate::models::User,
            // repo models
            crate::models::EntryKind,
            crate::models::TreeEntry,
            crate::models::CommitInfo,
            crate::models::Branch,
            // common error response
            crate::models::ErrorResponse
        )
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "repos", description = "Repository management"),
        (name = "git", description = "Git data browsing")
    ),
    modifiers(
        &SecurityAddon
    )
)]
struct ApiDoc;
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
        use utoipa::openapi::Components;

        let bearer = SecurityScheme::Http(
            HttpBuilder::new()
                .scheme(HttpAuthScheme::Bearer)
                .bearer_format("JWT")
                .build(),
        );

        let mut components = openapi.components.take().unwrap_or_else(Components::new);
        components.add_security_scheme("bearerAuth", bearer);
        openapi.components = Some(components);
    }
}

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
                    .url("/api/docs/openapi.json", ApiDoc::openapi()),
            )
            .configure(frontend::config)
            .configure(|x| actix_git_router::<WithAuth>(x))
    })
        .bind(bind_addr)?
        .run()
        .await
}