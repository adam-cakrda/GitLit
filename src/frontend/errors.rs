use actix_web::{HttpResponse, http::StatusCode};
use maud::{html, DOCTYPE};
use crate::frontend::{components, SERVE_PATH};

pub fn render_error_page(title: &str, heading: &str, message: &str, code: StatusCode) -> HttpResponse {
    let page = html! {
        (DOCTYPE)
        html lang="en" {
            (components::head(title, html!{
                link rel="stylesheet" href=(SERVE_PATH.to_string() + "/auth.css") {}
            }))
            (components::body(html!{
                main class="auth-container" {
                    div class="auth-card" {
                        (components::alert(components::AlertKind::Warning, message))
                        h1 { (heading) }
                        p { (message) }
                        div style="margin-top:1rem;" {
                            a href="/" class="auth-btn" { "Go Home" }
                        }
                    }
                }
            }, None))
        }
    };
    HttpResponse::build(code)
        .content_type("text/html; charset=utf-8")
        .body(page.into_string())
}

pub async fn not_found() -> HttpResponse {
    render_error_page("404 - Page Not Found", "Page not found", "The page you are looking for does not exist.", StatusCode::NOT_FOUND)
}

pub async fn internal_error() -> HttpResponse {
    render_error_page("500 - Server Error", "Something went wrong", "An unexpected error occurred. Please try again later.", StatusCode::INTERNAL_SERVER_ERROR)
}
