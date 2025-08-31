use actix_web::web;
mod index;
mod components;

pub use index::*;
pub use components::*;
use actix_files::Files;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .service(index)
        .service(Files::new("/", "./public").prefer_utf8(true));

}