use actix_web::{get, web, HttpRequest, Responder};
use maud::{html, Markup, DOCTYPE};
use mongodb::bson::doc;

use crate::db::Database;
use crate::frontend::components;
use crate::frontend::repo::utils::{page_shell};
use crate::frontend::SERVE_PATH;
use crate::models::*;
use crate::api::service;

fn profile_head() -> Markup {
    html! {
        link rel="stylesheet" href=(SERVE_PATH.to_string() + "/main.css") {}
        link rel="stylesheet" href=(SERVE_PATH.to_string() + "/profile.css") {}
    }
}

fn profile_page(user: &User, repos: &[Repository], requester: Option<&str>) -> Markup {
    let display = if user.display_name.is_empty() { &user.username } else { &user.display_name };
    page_shell(
        &format!("{} ({}) · GitLit", display, user.username),
        html! {
            main class="container profile-page" {
                div class="subheader" {
                    div class="subheader-left" {
                        h1 { (display) }
                        p class="muted" { "@" (user.username) }
                    }
                }
                section class="profile-sidebar" {
                    div class="avatar" {
                        @if let Some(url) = &user.avatar_url { img src=(url) alt="avatar" {} }
                        @else { div class="avatar-placeholder" { (user.username.chars().next().unwrap_or('?').to_uppercase()) } }
                    }
                    div class="meta" {
                        p { strong { "Member since: " } (user.created_at.to_string()) }
                        @if let Some(name) = requester { p { "Signed in as " (name) } }
                    }
                }
                section class="profile-content" {
                    div class="section-header" { h2 { "Repositories" } }
                    @if repos.is_empty() {
                        p class="muted" { "No repositories yet." }
                    } @else {
                        ul class="repo-list" {
                            @for r in repos {
                                li class="repo-item" {
                                    div class="repo-primary" {
                                        a class="repo-name" href={(format!("/{}/{}", user.username, r.name))} { (r.name.clone()) }
                                        @if !r.description.is_empty() { p class="repo-desc" { (r.description.clone()) } }
                                    }
                                    div class="repo-meta" {
                                        @if r.is_private { span class="badge" { "Private" } } @else { span class="badge" { "Public" } }
                                        span class="muted" { "Updated " (r.updated_at.to_string()) }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        requester,
    )
}

async fn requester_display(db: &Database, req: &HttpRequest) -> Option<String> {
    match crate::frontend::token_from_req(req) {
        Some(token) => match service::get_user_id_from_token(db, token).await {
            Ok(user_id) => {
                match db.users.find_one(doc!{ "_id": &user_id }).await {
                    Ok(Some(u)) => Some(u.display_name),
                    _ => None,
                }
            }
            Err(_) => None,
        },
        None => None,
    }
}

#[get("/{username}")]
pub async fn user_profile(
    db: web::Data<Database>,
    req: HttpRequest,
    path: web::Path<(String,)>,
) -> actix_web::Result<impl Responder> {
    let username = path.into_inner().0;

    let reserved = [
        "static", "login", "register", "logout", "api", "new",
    ];
    if reserved.contains(&username.as_str()) {
        return Err(actix_web::error::ErrorNotFound("not found"));
    }

    let user = db
        .find_user_by_login(&username)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or_else(|| actix_web::error::ErrorNotFound("user not found"))?;

    let requester_id = match crate::frontend::token_from_req(&req) {
        Some(tok) => service::get_user_id_from_token(&db, tok).await.ok(),
        None => None,
    };
    let filter = if requester_id == Some(user._id) {
        doc!{ "user": &user._id }
    } else {
        doc!{ "user": &user._id, "is_private": false }
    };

    use futures_util::TryStreamExt;
    let cursor = db.repositories.find(filter).sort(doc!{ "updated_at": -1 }).await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    let repos: Vec<Repository> = cursor.try_collect().await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let requester_name = requester_display(&db, &req).await;

    let markup = html! {
        (DOCTYPE)
        html { (components::head(&format!("{} · GitLit", user.username), profile_head()))
               (components::body(profile_page(&user, &repos, requester_name.as_deref()), requester_name.as_deref())) }
    };

    Ok(actix_web::HttpResponse::Ok().content_type("text/html; charset=utf-8").body(markup.into_string()))
}
