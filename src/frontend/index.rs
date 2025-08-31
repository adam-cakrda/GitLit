use actix_web::{get, App, HttpServer, Result, web};
use maud::{DOCTYPE, html, Markup};
use crate::frontend::components;

use crate::db::Database;
use mongodb::bson::{doc, oid::ObjectId};
use futures_util::TryStreamExt;
use std::collections::HashMap;

#[get("/")]
pub async fn index(db: web::Data<Database>) -> Result<Markup> {
    let filter = doc! { "is_private": false };
    let sort_doc = doc! { "created_at": -1 };
    let cursor = db.repositories
        .find(filter)
        .sort(sort_doc)
        .limit(12)
        .await
        .expect("failed to query repositories");
    let repos: Vec<crate::models::Repository> = cursor.try_collect().await.expect("failed to collect repos");

    let mut owner_ids: Vec<ObjectId> = Vec::new();
    for r in &repos {
        if !owner_ids.contains(&r.user) {
            owner_ids.push(r.user);
        }
    }
    let mut usernames: HashMap<ObjectId, String> = HashMap::new();
    if !owner_ids.is_empty() {
        let user_cursor = db.users
            .find(doc! { "_id": { "$in": &owner_ids } })
            .await
            .expect("failed to query users");
        let users: Vec<crate::models::User> = user_cursor.try_collect().await.expect("failed to collect users");
        for u in users {
            usernames.insert(u._id, u.username);
        }
    }

    Ok(html! {
        (DOCTYPE)
        html lang="en" {
            (components::head("GitLit", html! {
                link rel="stylesheet" href="home.css" {}
            }))

            (components::body(html! {
                section class="hero" {
                    h1 class="title" id="title" { "Fast, minimal and open-source git hosting." }
                    p class="subtitle" id="subtitle" { "GitLit is a open-source, Rust-powered alternative to GitHub. Built on " code { "actix-web" } " with server-side rendering for speed, simplicity, and predictability" }
                    div class="cta" {
                        a class="btn" href="/adam-cakrda/gitlit" { "Get Started" }
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
            }))
        }
    })
}