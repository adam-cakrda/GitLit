use actix_web::web;
mod index;
mod components;
mod auth;
mod repo;

use index::*;
use actix_files::Files;
use actix_web::HttpRequest;
use once_cell::sync::Lazy;
use std::env;
use actix_web::web::service;
pub static SERVE_PATH: Lazy<String> = Lazy::new(|| {
    env::var("SERVE_FILES_PATH").unwrap_or_else(|_| "/static".to_string())
});

pub fn token_from_req(req: &HttpRequest) -> Option<String> {
    req.cookie("token").map(|c| c.value().to_string())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .service(Files::new(SERVE_PATH.to_string().as_str(), "./public").prefer_utf8(true))

        .service(index)

        .service(auth::get_login)
        .service(auth::post_login)
        .service(auth::get_register)
        .service(auth::post_register)
        .service(auth::post_logout)

        .service(repo::index)
        .service(repo::tree)
        .service(repo::tree_at_path)
        .service(repo::blob)
        .service(repo::commits)
        .service(repo::new::get)
        .service(repo::new::post);

}