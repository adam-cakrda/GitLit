use actix_web::web;
mod index;
mod components;
mod auth;

pub use index::*;
use actix_files::Files;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .service(index)
        .service(auth::get_login)
        .service(auth::post_login)
        .service(auth::get_register)
        .service(auth::post_register)
        .service(auth::post_logout)
        .service(Files::new("/", "./public").prefer_utf8(true));

}