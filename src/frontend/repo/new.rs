use actix_web::{get, post, web, HttpRequest, HttpResponse, Result};
use maud::{html, Markup};

use crate::db::Database;
use crate::frontend::{components, SERVE_PATH};
use crate::frontend::repo::utils;
use crate::api::service;

#[derive(serde::Deserialize)]
pub struct NewRepoForm {
    pub name: String,
    pub description: Option<String>,
    pub visibility: Option<String>,
    pub init_readme: Option<String>, //TODO: INIT README
}

#[post("/new")]
pub async fn post(db: web::Data<Database>, req: HttpRequest, form: web::Form<NewRepoForm>) -> Result<HttpResponse> {
    let token = match crate::frontend::token_from_req(&req) {
        Some(t) => t,
        None => {
            return Ok(HttpResponse::SeeOther()
                .append_header((actix_web::http::header::LOCATION, "/login"))
                .finish());
        }
    };

    let user_id = match service::get_user_id_from_token(&db, token).await {
        Ok(uid) => uid,
        Err(_) => {
            return Ok(HttpResponse::SeeOther()
                .append_header((actix_web::http::header::LOCATION, "/login"))
                .finish());
        }
    };

    let is_private = match form.visibility.as_deref() {
        Some("private") => Some(true),
        _ => Some(false),
    };

    let payload = crate::models::CreateRepoRequest {
        name: form.name.trim().to_string(),
        description: form.description.clone().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        is_private,
    };

    match service::repo_create(&db, user_id, payload).await {
        Ok(repo) => {
            let owner = service::username_by_id(&db, &user_id).await.ok().flatten().unwrap_or_else(|| "me".to_string());
            let location = format!("/{}/{}", owner, repo.name);
            Ok(HttpResponse::SeeOther()
                .append_header((actix_web::http::header::LOCATION, location))
                .finish())
        }
        Err(_) => {
            //TODO: Error page
            Ok(HttpResponse::SeeOther()
                .append_header((actix_web::http::header::LOCATION, "/new"))
                .finish())
        }
    }
}

#[get("/new")]
pub async fn get(db: web::Data<Database>, req: HttpRequest) -> Result<Markup> {
    let user_display = utils::token_display(&db, &req).await;

    Ok(html! {
        (maud::DOCTYPE)
        html lang="en" {
            (components::head("New repository - GitLit", html! {
                link rel="stylesheet" href=(SERVE_PATH.to_string() + "/new.css") {}
            }))
            (components::body(html! {
                main class="new-repo-container" {
                    section class="new-repo-card" {
                        header class="new-repo-header" {
                            h1 { "Create a new repository" }
                            p class="sub" { "A repository contains all project files, including the revision history." }
                        }

                        form class="new-repo-form" method="post" action="/new" {
                            fieldset class="form-group" {
                                label for="repo-name" { "Repository name" }
                                input type="text" id="repo-name" name="name" placeholder="my-awesome-project" required {}
                                p class="help" { "Great repository names are short and memorable." }
                            }

                            fieldset class="form-group" {
                                label for="repo-desc" { "Description " span class="muted" { "(optional)" } }
                                textarea id="repo-desc" name="description" rows="3" placeholder="A short description of your repository" {}
                            }

                            fieldset class="form-group" {
                                label { "Visibility" }
                                div class="visibility" {
                                    label class="radio" {
                                        input type="radio" name="visibility" value="public" checked {}
                                        span class="title" { "Public" }
                                        span class="detail" { "Anyone on the internet can see this repository." }
                                    }
                                    label class="radio" {
                                        input type="radio" name="visibility" value="private" {}
                                        span class="title" { "Private" }
                                        span class="detail" { "You choose who can see and commit to this repository." }
                                    }
                                }
                            }

                            fieldset class="form-group init" {
                                label class="checkbox" {
                                    input type="checkbox" name="init_readme" {}
                                    span { "Initialize this repository with a README" }
                                }
                            }

                            div class="actions" {
                                button type="submit" class="create-btn" { "Create repository" }
                            }
                        }
                    }
                }
            }, user_display.as_deref()))
        }
    })
}