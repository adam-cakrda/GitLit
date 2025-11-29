use actix_web::{Result, web, HttpRequest, get};
use crate::db::Database;
use crate::frontend::components;
use crate::frontend::repo::utils;
use crate::repo;
use maud::{Markup, html};
use crate::frontend::SERVE_PATH;

#[get("/{username}/{reponame}")]
pub async fn index(
    db: web::Data<Database>,
    req: HttpRequest,
    path: web::Path<(String, String)>,
) -> Result<Markup> {
    let (username, reponame) = path.into_inner();
    let (owner, repo) = utils::resolve_owner_repo(&db, &username, &reponame).await?;
    let user_display = utils::token_display(&db, &req).await;
    let conn = req.connection_info();
    let clone_url = format!("{}://{}/{}/{}.git", conn.scheme(), conn.host(), owner.username, repo.name);

    let default = utils::default_ref(&db, &owner, &repo).await;

    let entries = repo::list_tree(&owner._id, &repo._id, &default, Some(&default), None)
        .await
        .unwrap_or_default();
    
    let branches = repo::list_branches(&owner._id, &repo._id).await.unwrap_or_default();
    let total_commits = repo::list_commits(&owner._id, &repo._id, &default, Some(&default), 0)
        .await
        .map(|v| v.len())
        .unwrap_or(0);

    let readme = async {
        let candidates = ["README.md", "Readme.md", "readme.md", "README.MD"];
        for c in candidates {
            if let Ok(bytes) = repo::get_file_content(&owner._id, &repo._id, &default, Some(&default), c).await {
                if let Some(markup) = utils::render_readme_html(&bytes) {
                    return Some(markup);
                }
            }
        }
        None
    }.await;

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
                                    (default.clone())
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
                                    li {
                                        hr {};
                                        a class="see-all-branches" href={(format!("/{}/{}/branches", owner.username, repo.name))} {
                                            "See all branches"
                                        }
                                    }
                                }
                            }
                            div class="commit-info" { "" }
                            a class="commits-btn" href={(format!("/{}/{}/commits/{}", owner.username, repo.name, default))} {
                                "Commits "
                                span class="badge" { (total_commits) }
                            }
                            div class="repo-actions" {
                                div class="code-menu" {
                                    button class="action-btn menu-trigger" type="button" { "Code" }
                                    div class="code-popup" {
                                        div class="clone-field" {
                                            input type="text" readonly value=(clone_url.clone()) {}
                                        }
                                        a class="download-zip" href={(format!("/api/v1/download?id={}&branch={}", repo._id, default))} { "Download as ZIP" }
                                    }
                                }
                            }
                        }
                        (utils::breadcrumbs(&owner.username, &repo.name, &default, None))
                        (utils::file_list(&owner.username, &repo.name, &default, None, &entries))
                    }

                    @if let Some(readme_html) = readme {
                        div class="content-viewer" {
                            div class="content-header" {
                                span class="content-title" { "📄 README.md" }
                            }
                            div class="content-body" {
                                div class="readme-content" {
                                    (readme_html)
                                }
                            }
                        }
                    }
                }
                div class="sidebar" {
                    div class="sidebar-section" {
                        div class="sidebar-header" { "About" }
                        div class="sidebar-content" {
                            div class="description" { (repo.description.clone()) }
                        }
                    }
                }
            }
        }
    };

    Ok(utils::page_shell(&format!("{} / {}", owner.username, repo.name), content, user_display.as_deref()))
}