use actix_web::{Result, web, HttpRequest, get};
use crate::db::Database;
use maud::{Markup, html};
use crate::frontend::components;
use crate::frontend::SERVE_PATH;
use crate::repo;
use crate::frontend::repo::utils;
async fn render_tree_page(
    db: &Database,
    req: &HttpRequest,
    username: String,
    reponame: String,
    rev_in: String,
    subpath: Option<String>,
) -> Result<Markup> {
    let (owner, repo) = utils::resolve_owner_repo(db, &username, &reponame).await?;
    let user_display = utils::token_display(db, req).await;

    let rev_owned = rev_in.clone();
    let (rev, branch_opt_owned): (String, Option<String>) = if utils::is_hex_hash(&rev_in) {
        (rev_owned, None)
    } else {
        (rev_owned, Some(rev_in))
    };
    let branch_opt = branch_opt_owned.as_deref();
    let path_opt = subpath.as_deref();

    let entries = repo::list_tree(&owner._id, &repo._id, &rev, branch_opt, path_opt)
        .await
        .unwrap_or_default();

    let branches = repo::list_branches(&owner._id, &repo._id).await.unwrap_or_default();
    let reference = branch_opt.unwrap_or(&rev);
    let total_commits = repo::list_commits(&owner._id, &repo._id, reference, branch_opt, 0)
        .await
        .map(|v| v.len())
        .unwrap_or(0);

    let content = html! {
        (components::repo_header(user_display.as_deref(), &owner.username, &repo.name, repo.is_private))
        div class="container" {
            div class="main-content" {
                div class="left-content" {
                    div class="file-explorer" {
                        div class="explorer-header" {
                            div class="branch-dropdown" {
                                button class="branch-btn" {
                                    img src=(SERVE_PATH.to_string() + "/branch.svg") alt="Branch" {}
                                    (reference)
                                }
                                ul class="branch-menu" {
                                    @for b in &branches {
                                        li {
                                            a href={(format!("/{}/{}/tree/{}", owner.username, repo.name, b.name))} {
                                                @if b.is_head { span class="badge-head" { "HEAD" } }
                                                (b.name.clone())
                                            }
                                        }
                                    }
                                }
                            }
                            div class="commit-info" { "" }
                            a class="commits-btn" href={(format!("/{}/{}/commits/{}", owner.username, repo.name, reference))} {
                                "Commits "
                                span class="badge" { (total_commits) }
                            }
                        }
                        (utils::breadcrumbs(&owner.username, &repo.name, reference, path_opt))
                        (utils::file_list(&owner.username, &repo.name, reference, path_opt, &entries))
                    }
                }
                div class="sidebar" { }
            }
        }
    };

    Ok(utils::page_shell(&format!("{} / {} - tree {}", owner.username, repo.name, rev), content, user_display.as_deref()))
}

#[get("/{username}/{reponame}/tree/{rev}")]
pub async fn tree(
    db: web::Data<Database>,
    req: HttpRequest,
    path: web::Path<(String, String, String)>,
) -> Result<Markup> {
    let (username, reponame, rev_in) = path.into_inner();
    render_tree_page(&db, &req, username, reponame, rev_in, None).await
}

#[get("/{username}/{reponame}/tree/{rev}/{path:.*}")]
pub async fn tree_at_path(
    db: web::Data<Database>,
    req: HttpRequest,
    path: web::Path<(String, String, String, String)>,
) -> Result<Markup> {
    let (username, reponame, rev_in, subpath) = path.into_inner();
    render_tree_page(&db, &req, username, reponame, rev_in, Some(subpath)).await
}