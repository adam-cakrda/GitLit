use actix_web::{get, web, HttpRequest, Result};
use maud::{html, Markup};
use crate::db::Database;
use crate::frontend::components;
use crate::frontend::repo::utils;
use crate::repo;

#[get("/{username}/{reponame}/commits/{rev}")]
pub async fn commits(
    db: web::Data<Database>,
    req: HttpRequest,
    path: web::Path<(String, String, String)>,
) -> Result<Markup> {
    let (username, reponame, rev_in) = path.into_inner();
    let (owner, repository) = utils::resolve_owner_repo(&db, &username, &reponame).await?;
    let user_display = utils::token_display(&db, &req).await;

    let (rev, branch_opt_owned): (String, Option<String>) = if utils::is_hex_hash(&rev_in) {
        (rev_in.clone(), None)
    } else {
        (rev_in.clone(), Some(rev_in))
    };
    let reference = branch_opt_owned.as_deref().unwrap_or(&rev);

    let branches = repo::list_branches(&owner._id, &repository._id).await.unwrap_or_default();

    let commits = repo::list_commits(&owner._id, &repository._id, reference, branch_opt_owned.as_deref(), 100)
        .await
        .unwrap_or_default();

    let is_hash_view = branch_opt_owned.is_none();

    let (detail_markup_opt, title_suffix) = if is_hash_view {
        let c_opt = commits.get(0);
        let details = if let Some(c) = c_opt {
            let diff_text = repo::commit_diff(&owner._id, &repository._id, &c.hash)
                .await
                .unwrap_or_else(|_| String::from("Failed to load diff."));
            Some(html! {
                div class="content-viewer" {
                    div class="content-header" {
                        span class="content-title" { "Commit " (c.hash.chars().take(12).collect::<String>()) }
                    }
                    div class="content-body" {
                        p class="commit-subject" { (&c.subject) }
                        p class="commit-meta" { "Author: " (&c.name) " <" (&c.email) "> • " (utils::format_time(c.timestamp_secs)) }
                        pre { code { (diff_text) } }
                    }
                }
            })
        } else {
            None
        };
        (details, format!("commit {}", reference))
    } else {
        (None, format!("branch {}", reference))
    };

    let content = html! {
        (components::repo_header(user_display.as_deref(), &owner.username, &repository.name, repository.is_private))
        div class="container" {
            div class="main-content" {
                div class="left-content" {
                    div class="file-explorer" {
                        div class="explorer-header" {
                            div class="branch-dropdown" {
                                button class="branch-btn" {
                                    (reference)
                                }
                                ul class="branch-menu" {
                                    @for b in &branches {
                                        li {
                                            a href={(format!("/{}/{}/commits/{}", owner.username, repository.name, b.name))} {
                                                @if b.is_head { span class="badge-head" { "HEAD" } }
                                                (b.name.clone())
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    @if let Some(dv) = detail_markup_opt {
                        (dv)
                    }
                    
                    @if !is_hash_view {
                        div class="content-viewer" {
                            div class="content-header" {
                                span class="content-title" {
                                    "Commits on " (reference)
                                }
                            }
                            div class="content-body commits-list" {
                                @if commits.is_empty() {
                                    p { "No commits found." }
                                } @else {
                                    ul class="commits-ul" {
                                        @for c in &commits {
                                            li class="commit-li" {
                                                a href={(format!("/{}/{}/commits/{}", owner.username, repository.name, c.hash))} class="commit-row" {
                                                    div class="commit-main" {
                                                        div class="commit-subject" { (&c.subject) }
                                                        div class="commit-meta" {
                                                            span class="commit-author" { (&c.name) }
                                                            " • "
                                                            span class="commit-time" { (utils::format_time(c.timestamp_secs)) }
                                                        }
                                                    }
                                                    div class="commit-side" {
                                                        span class="commit-hash" { (&c.hash.chars().take(7).collect::<String>()) }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div class="sidebar" { }
            }
        }
    };

    Ok(utils::page_shell(
        &format!("{}/{} - {}", owner.username, repository.name, title_suffix),
        content,
        user_display.as_deref(),
    ))
}