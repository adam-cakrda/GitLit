use actix_web::{get, post, web, HttpRequest, HttpResponse, Result};
use crate::db::Database;
use crate::frontend::repo::utils;
use crate::repo;
use maud::{html, Markup};

#[get("/{username}/{reponame}/branches")]
pub async fn list(
    db: web::Data<Database>,
    req: HttpRequest,
    path: web::Path<(String, String)>,
) -> Result<Markup> {
    let (username, reponame) = path.into_inner();
    let (owner, repo) = utils::resolve_owner_repo(&db, &username, &reponame).await?;
    let user_display = utils::token_display(&db, &req).await;
    let branches = repo::list_branches(&owner._id, &repo._id).await.unwrap_or_default();
    let is_owner = user_display.as_deref() == Some(&owner.username);

    let content = html! {
        (crate::frontend::components::repo_header(user_display.as_deref(), &owner.username, &repo.name, repo.is_private))
        div class="container" {
            h2 { "Branches" }
            ul class="branch-list" {
                @for b in &branches {
                    li {
                        a href={(format!("/{}/{}/tree/{}", owner.username, repo.name, b.name))} {
                            (b.name.clone())
                        }
                        @if is_owner && !b.is_head {
                            a href={(format!("/{}/{}/branches/delete/{}", owner.username, repo.name, b.name))} {
                                button type="button" class="branch-delete-btn" { "Delete" }
                            }
                        }
                        @if b.is_head {
                            span class="badge-head" { "HEAD" }
                        }
                    }
                }
            }
        }
    };
    Ok(utils::page_shell(&format!("{} / {} / branches", owner.username, repo.name), content, user_display.as_deref()))
}

#[get("/{username}/{reponame}/branches/delete/{branch}")]
pub async fn confirm_delete(
    db: web::Data<Database>,
    req: HttpRequest,
    path: web::Path<(String, String, String)>,
) -> Result<Markup> {
    let (username, reponame, branch) = path.into_inner();
    let (owner, repo) = utils::resolve_owner_repo(&db, &username, &reponame).await?;
    let user_display = utils::token_display(&db, &req).await;
    let is_owner = user_display.as_deref() == Some(&owner.username);

    if !is_owner {
        return Ok(html! { p { "Forbidden" } });
    }

    let content = html! {
    (crate::frontend::components::repo_header(user_display.as_deref(), &owner.username, &repo.name, repo.is_private))
        div class="container" {
            div class="confirm-delete-box" {
                h2 { "Delete branch: " (branch) }
                p { "To confirm deletion, type the branch name below:" }
                form method="post" action={(format!("/{}/{}/branches/delete/{}", owner.username, repo.name, branch))} {
                    input type="text" name="confirm_name" placeholder="Branch name" required {}
                    input type="hidden" name="branch" value=(branch.clone()) {}
                    button type="submit" class="branch-delete-btn" { "Delete branch" }
                }
                a href={(format!("/{}/{}/branches", owner.username, repo.name))} { "Cancel" }
            }
        }
    };

    Ok(utils::page_shell(&format!("Delete branch {}", branch), content, user_display.as_deref()))
}

#[post("/{username}/{reponame}/branches/delete/{branch}")]
pub async fn do_delete(
    db: web::Data<Database>,
    req: HttpRequest,
    path: web::Path<(String, String, String)>,
    form: web::Form<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse> {
    let (username, reponame, branch) = path.into_inner();
    let (owner, repo) = utils::resolve_owner_repo(&db, &username, &reponame).await?;
    let user_display = utils::token_display(&db, &req).await;
    let is_owner = user_display.as_deref() == Some(&owner.username);

    if !is_owner {
        return Ok(HttpResponse::Forbidden().body("Forbidden"));
    }

    if let Some(confirm) = form.get("confirm_name") {
        if confirm == &branch {
            let _ = repo::delete_branch(&owner._id, &repo._id, &branch).await;
            return Ok(HttpResponse::SeeOther()
                .append_header(("Location", format!("/{}/{}/branches", owner.username, repo.name)))
                .finish());
        }
    }
    // TODO: error page
    Ok(HttpResponse::SeeOther()
        .append_header(("Location", format!("/{}/{}/branches/delete/{}", owner.username, repo.name, branch)))
        .finish())
}