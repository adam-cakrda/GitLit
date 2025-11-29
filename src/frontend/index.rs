
use actix_web::{get, Result, web, HttpRequest};
use maud::{DOCTYPE, html, Markup};

use crate::db::Database;
use std::collections::HashMap;
use crate::api::service;
use crate::models::ReposQuery;
use crate::frontend;
use crate::frontend::{components, SERVE_PATH};
use crate::db::Repository;
use bson::oid::ObjectId;

#[get("/")]
pub async fn index(db: web::Data<Database>, req: HttpRequest) -> Result<Markup> {
    let repos_all = service::repo_list(
        &db,
        None,
        ReposQuery {
            owner: None,
            filter: Some("newest".to_string()),
            q: None,
        },
    )
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let repos: Vec<Repository> = repos_all.into_iter().take(12).collect();

    let mut owner_ids: Vec<ObjectId> = Vec::new();
    for r in &repos {
        if !owner_ids.contains(&r.user) {
            owner_ids.push(r.user.clone());
        }
    }

    let mut usernames: HashMap<ObjectId, String> = HashMap::new();
    for uid in &owner_ids {
        match service::username_by_id(&db, uid).await {
            Ok(Some(name)) => {
                usernames.insert(uid.clone(), name);
            }
            Ok(None) => {}
            Err(e) => {
                return Err(actix_web::error::ErrorInternalServerError(e));
            }
        }
    }

    let user_display: Option<String> = match frontend::token_from_req(&req) {
        Some(token) => {
            match service::get_user_id_from_token(&db, token).await {
                Ok(user_id) => {
                    match db.find_user_by_id(&user_id).await {
                        Ok(Some(u)) => Some(u.display_name),
                        _ => None,
                    }
                }
                Err(_) => None,
            }
        }
        None => None,
    };

    Ok(html! {
        (DOCTYPE)
        html lang="en" {
            (components::head("GitLit", html! {
                link rel="stylesheet" href=(SERVE_PATH.to_string() + "/home.css") {}
            }))

            (components::body(html! {
                main {
                    section class="hero" {
                        h1 class="title" id="title" { "Fast, minimal and open-source git hosting." }
                        p class="subtitle" id="subtitle" { "GitLit is a open-source, Rust-powered alternative to GitHub. Built on " code { "actix-web" } " with server-side rendering for speed, simplicity, and predictability" }
                        div class="cta" {
                            a class="btn" href="/adam-cakrda/GitLit" { "Get Started" }
                        }
                    }

                    section class="repos" {
                        h2 class="repos-title" { "Newest Repositories" }
                        div class="repo-grid" {
                            @for r in &repos {
                                @let owner = usernames.get(&r.user).map(|s| s.as_str()).unwrap_or("unknown");
                                @let href = format!("/{}/{}", owner, r.name);
                                div class="repo-card" {
                                    div class="repo-header" {
                                        a href={(href.clone())} class="name" {
                                            (owner) " / " span class="repo-name" { (&r.name) }
                                        }
                                        span class="repo-visibility" { (if r.is_private { "Private" } else { "Public" }) }
                                    }
                                    p class="repo-desc" { (r.description.clone()) }
                                }
                            }
                        }
                    }
                }
            }, user_display.as_deref()))
        }
    })
}