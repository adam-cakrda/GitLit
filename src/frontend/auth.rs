use actix_web::{get, post, web, HttpRequest, HttpResponse, Result};
use actix_web::http::header::LOCATION;
use actix_web::cookie::{Cookie, SameSite, time::Duration};
use maud::{DOCTYPE, html};
use crate::db::Database;
use crate::api::service;
use crate::frontend::components;
use crate::frontend::SERVE_PATH;
use std::env;
use crate::errors::AuthError;

#[derive(serde::Deserialize)]
struct ErrorQuery { error: Option<String> }

#[derive(serde::Deserialize)]
pub struct LoginForm {
    pub login: String,
    pub password: String,
}

#[derive(serde::Deserialize)]
pub struct RegisterForm {
    pub username: String,
    pub email: String,
    pub password: String,
}

async fn redirect_if_logged(db: &Database, req: &HttpRequest) -> Option<HttpResponse> {
    if let Some(c) = req.cookie("token") {
        let token = c.value().to_string();
        if service::get_user_id_from_token(db, token).await.is_ok() {
            return Some(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/"))
                .finish());
        }
    }
    None
}

#[get("/login")]
pub async fn get_login(db: web::Data<Database>, req: HttpRequest, query: web::Query<ErrorQuery>) -> Result<HttpResponse> {
    if let Some(resp) = redirect_if_logged(&db, &req).await {
        return Ok(resp);
    }
    let page = html! {
        (DOCTYPE)
        html lang="en" {
            (components::head("Login - GitLit", html! {
                link rel="stylesheet" href=(SERVE_PATH.to_string() + "/auth.css") {}
            }))
            (components::body(html! {
                main class="auth-container" {
                    div class="auth-card" {
                        div class="auth-header" {
                            h1 { "Log in" }
                        }
                        @if let Some(err) = &query.error {
                            (components::alert(components::AlertKind::Error, err))
                        }
                        form class="auth-form" method="post" action="/login" {
                            div class="form-group" {
                                label for="username" { "Username or email" }
                                input type="text" id="username" name="login" required {}
                            }
                            div class="form-group" {
                                label for="password" { "Password" }
                                input type="password" id="password" name="password" required {}
                            }
                            div class="form-options" {
                                label class="checkbox-label" {
                                    input type="checkbox" name="remember" {}
                                    span class="checkmark" {}
                                    "Remember me"
                                }
                                a href="/forgot-password" class="forgot-link" { "Forgot password?" }
                            }
                            button type="submit" class="auth-btn" { "Sign in" }
                        }
                        div class="auth-footer" {
                            p { "New to GitLit? " a href="/register" { "Create an account" } }
                        }
                    }
                }

            }, None))
        }
    };

    Ok(HttpResponse::Ok().content_type("text/html; charset=utf-8").body(page.into_string()))
}

#[post("/login")]
pub async fn post_login(db: web::Data<Database>, form: web::Form<LoginForm>) -> Result<HttpResponse> {
    match service::auth_login(&db, form.login.clone(), form.password.clone()).await {
        Ok(token) => {
            let cookie = Cookie::build("token", token)
                .http_only(true)
                .same_site(SameSite::Lax)
                .path("/")
                .max_age(Duration::seconds(24 * 60 * 60))
                .finish();

            Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/"))
                .cookie(cookie)
                .finish())
        }
        Err(_) => {
            Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/login?error=Invalid%20credentials"))
                .finish())
        }
    }
}

#[get("/register")]
pub async fn get_register(db: web::Data<Database>, req: HttpRequest, query: web::Query<ErrorQuery>) -> Result<HttpResponse> {
    if let Some(resp) = redirect_if_logged(&db, &req).await {
        return Ok(resp);
    }
    let page = html! {
        (DOCTYPE)
        html lang="en" {
            (components::head("Register - GitLit", html! {
                link rel="stylesheet" href=(SERVE_PATH.to_string() + "/auth.css") {}
            }))
            @if env::var("ALLOW_REGISTER").unwrap_or_else(|_| "false".to_string()) != "true" {
                (components::body(html! {
                    main class="auth-container" { div class="auth-card" {
                        (components::alert(components::AlertKind::Warning, &AuthError::RegistrationDisabled.to_string()))
                    }}
                }, None))
            } @else {
                (components::body(html! {
                    main class="auth-container" {
                        div class="auth-card" {
                            div class="auth-header" {
                                h1 { "Create your account" }
                            }
                            @if let Some(err) = &query.error {
                                (components::alert(components::AlertKind::Error, err))
                            }
                            form class="auth-form" method="post" action="/register" {
                                div class="form-group" {
                                    label for="username" { "Username" }
                                    input type="text" id="username" name="username" required {}
                                    div class="input-help" { "Choose a unique username for your profile" }
                                }
                                div class="form-group" {
                                    label for="email" { "Email address" }
                                    input type="email" id="email" name="email" required {}
                                }
                                div class="form-group" {
                                    label for="password" { "Password" }
                                    input type="password" id="password" name="password" required {}
                                    div class="input-help" { "Must be at least 8 characters long" }
                                }
                                div class="form-group" {
                                    label for="confirm-password" { "Confirm password" }
                                    input type="password" id="confirm-password" name="confirm-password" required {}
                                }
                                div class="form-options" {
                                    label class="checkbox-label" {
                                        input type="checkbox" name="terms" required {}
                                        span class="checkmark" {}
                                        "I agree to the "
                                        a href="/terms" { "Terms of Service" }
                                        " and "
                                        a href="/privacy" { "Privacy Policy" }
                                    }
                                }
                                button type="submit" class="auth-btn" { "Create account" }
                            }
                            div class="auth-footer" {
                                p { "Already have an account? " a href="/login" { "Sign in" } }
                            }
                        }
                    }

                }, None))
            }
        }
    };
    Ok(HttpResponse::Ok().content_type("text/html; charset=utf-8").body(page.into_string()))
}

#[post("/register")]
pub async fn post_register(db: web::Data<Database>, form: web::Form<RegisterForm>) -> Result<HttpResponse> {
    match service::auth_register(&db, form.username.clone(), form.email.clone(), form.password.clone()).await {
        Ok(()) => {
            Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/login"))
                .finish())
        }
        Err(_) => {
            Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/register?error=Registration%20failed"))
                .finish())
        }
    }
}

#[post("/logout")]
pub async fn post_logout(db: web::Data<Database>, req: HttpRequest) -> Result<HttpResponse> {
    if let Some(c) = req.cookie("token") {
        let token = c.value().to_string();
        let _ = service::auth_logout(&db, token).await;
    }

    let removal = Cookie::build("token", "")
        .path("/")
        .same_site(SameSite::Lax)
        .http_only(true)
        .max_age(Duration::seconds(0))
        .finish();

    Ok(HttpResponse::SeeOther()
        .insert_header((LOCATION, "/"))
        .cookie(removal)
        .finish())
}