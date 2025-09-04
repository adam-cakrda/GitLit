use actix_web::{get, web, HttpRequest, Result};
use maud::{html, Markup, DOCTYPE, PreEscaped};
use crate::db::Database;
use crate::frontend::components;
use crate::api::service;
use mongodb::bson::doc;
use crate::models::{EntryKind, TreeEntry};
use crate::repo as git_repo;
use crate::frontend::SERVE_PATH;

fn is_hex_hash(s: &str) -> bool {
    let len = s.len();
    if !(7..=64).contains(&len) { return false; }
    s.chars().all(|c| c.is_ascii_hexdigit())
}

async fn token_display(db: &Database, req: &HttpRequest) -> Option<String> {
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

async fn resolve_owner_repo(
    db: &Database,
    username: &str,
    reponame: &str,
) -> actix_web::Result<(crate::models::User, crate::models::Repository)> {
    let owner = db
        .find_user_by_login(username)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or_else(|| actix_web::error::ErrorNotFound("owner not found"))?;

    let repo = db
        .repositories
        .find_one(doc! { "user": &owner._id, "name": reponame })
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or_else(|| actix_web::error::ErrorNotFound("repository not found"))?;

    Ok((owner, repo))
}

async fn default_ref(db: &Database, owner: &crate::models::User, repo: &crate::models::Repository) -> String {
    match git_repo::list_branches(&owner._id, &repo._id).await {
        Ok(list) => {
            if let Some(h) = list.iter().find(|b| b.is_head) {
                h.name.clone()
            } else if let Some(first) = list.first() {
                first.name.clone()
            } else {
                "HEAD".to_string()
            }
        }
        Err(_) => "HEAD".to_string(),
    }
}

fn entry_icon_and_name(e: &TreeEntry) -> (Markup, Markup) {
    match e.kind {
        EntryKind::Tree => (html! { span class="icon-folder file-icon" {} }, html! { span class="file-name folder" { (&e.path) } }),
        _ => (html! { span class="icon-file file-icon" {} }, html! { span class="file-name" { (&e.path) } }),
    }
}

fn render_readme_html(md_bytes: &[u8]) -> Option<Markup> {
    let md = match std::str::from_utf8(md_bytes) {
        Ok(s) => s,
        Err(_) => return None,
    };
    let mut html_buf = String::new();
    let parser = pulldown_cmark::Parser::new_ext(md, pulldown_cmark::Options::all());
    pulldown_cmark::html::push_html(&mut html_buf, parser);
    let clean = ammonia::Builder::default().clean(&html_buf).to_string();
    Some(PreEscaped(clean).into())
}

fn repo_header(owner_slug: &str, repo_slug: &str, is_private: bool) -> Markup {
    let visibility = if is_private { "Private" } else { "Public" };
    html! {
        header {
            div class="header-top" {
                img src=(SERVE_PATH.to_string() + "/gitlit.svg") alt="logo" {}
                div class="name" {
                    a href={(format!("/{}", owner_slug))} { (owner_slug) }
                    " / "
                    a href={(format!("/{}/{}", owner_slug, repo_slug))} class="white" { (repo_slug) }
                }
                div class="repo-visibility" { (visibility) }
            }
            nav class="repo-nav" {
                a class="nav-item active" {
                    img src=(SERVE_PATH.to_string() + "/code.svg") alt="code" class="icon-branch" {}
                    "Code"
                }
                a class="nav-item" { "Settings" }
            }
        }
    }
}

fn file_list(
    owner_slug: &str,
    repo_slug: &str,
    entries: &[TreeEntry],
) -> Markup {
    html! {
        ul class="file-list" {
            @for e in entries {
                @let (icon, name_markup) = entry_icon_and_name(e);
                @match e.kind {
                    EntryKind::Tree => {
                        li class="file-item" {
                            (icon)
                            a href={(format!("/{}/{}/tree/{}", owner_slug, repo_slug, e.path))} { (name_markup) }
                            span class="file-message" { "" }
                            span class="file-time" { "" }
                        }
                    }
                    EntryKind::Blob | EntryKind::Commit | EntryKind::Other(_) => {
                        li class="file-item" {
                            (icon)
                            a href={(format!("/{}/{}/blob/{}", owner_slug, repo_slug, e.path))} { (name_markup) }
                            span class="file-message" { @if let Some(sz) = e.size { (format!("{} B", sz)) } }
                            span class="file-time" { "" }
                        }
                    }
                }
            }
        }
    }
}

fn page_shell(page_title: &str, inner: Markup, user_display: Option<&str>) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            (components::head(page_title, html! {
                link rel="stylesheet" href=(SERVE_PATH.to_string() + "/repo.css") {}
            }))
            (components::body(inner, user_display))
        }
    }
}

#[get("/{username}/{reponame}")]
pub async fn repo_overview(
    db: web::Data<Database>,
    req: HttpRequest,
    path: web::Path<(String, String)>,
) -> Result<Markup> {
    let (username, reponame) = path.into_inner();
    let (owner, repo) = resolve_owner_repo(&db, &username, &reponame).await?;
    let user_display = token_display(&db, &req).await;

    let default = default_ref(&db, &owner, &repo).await;

    let entries = git_repo::list_tree(&owner._id, &repo._id, &default, Some(&default), None)
        .await
        .unwrap_or_default();

    // Try README.md in root
    let readme = async {
        let candidates = ["README.md", "Readme.md", "readme.md", "README.MD"];
        for c in candidates {
            if let Ok(bytes) = git_repo::get_file_content(&owner._id, &repo._id, &default, Some(&default), c).await {
                if let Some(markup) = render_readme_html(&bytes) {
                    return Some(markup);
                }
            }
        }
        None
    }.await;

    let content = html! {
        (repo_header(&owner.username, &repo.name, repo.is_private))
        div class="container" {
            div class="main-content" {
                div class="left-content" {
                    div class="file-explorer" {
                        div class="explorer-header" {
                            button class="branch-btn" {
                                img src="/branch.svg" alt="Branch" {}
                                (default.clone())
                            }
                            div class="commit-info" { "" }
                        }
                        (file_list(&owner.username, &repo.name, &entries))
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

    Ok(page_shell(&format!("{} / {}", owner.username, repo.name), content, user_display.as_deref()))
}

#[get("/{username}/{reponame}/tree/{rev:.*}")]
pub async fn repo_tree(
    db: web::Data<Database>,
    req: HttpRequest,
    path: web::Path<(String, String, String)>,
) -> Result<Markup> {
    let (username, reponame, rev_in) = path.into_inner();
    let (owner, repo) = resolve_owner_repo(&db, &username, &reponame).await?;
    let user_display = token_display(&db, &req).await;

    let (rev, branch_opt) = if is_hex_hash(&rev_in) { (rev_in.clone(), None) } else { (rev_in.clone(), Some(rev_in.as_str())) };

    let entries = git_repo::list_tree(&owner._id, &repo._id, &rev, branch_opt, None)
        .await
        .unwrap_or_default();

    let content = html! {
        (repo_header(&owner.username, &repo.name, repo.is_private))
        div class="container" {
            div class="main-content" {
                div class="left-content" {
                    div class="file-explorer" {
                        div class="explorer-header" {
                            button class="branch-btn" {
                                img src=(SERVE_PATH.to_string() + "/branch.svg") alt="Branch" {}
                                (branch_opt.unwrap_or(&rev))
                            }
                            div class="commit-info" { "" }
                        }
                        (file_list(&owner.username, &repo.name, &entries))
                    }
                }
                div class="sidebar" { }
            }
        }
    };

    Ok(page_shell(&format!("{} / {} - tree {}", owner.username, repo.name, rev), content, user_display.as_deref()))
}

#[get("/{username}/{reponame}/blob/{path:.*}")]
pub async fn repo_blob(
    db: web::Data<Database>,
    req: HttpRequest,
    path: web::Path<(String, String, String)>,
) -> Result<Markup> {
    let (username, reponame, blob_path) = path.into_inner();
    let (owner, repo) = resolve_owner_repo(&db, &username, &reponame).await?;
    let user_display = token_display(&db, &req).await;

    // Use default ref when blob route has no explicit branch in URL
    let default = default_ref(&db, &owner, &repo).await;

    let content_bytes = git_repo::get_file_content(&owner._id, &repo._id, &default, Some(&default), &blob_path)
        .await
        .map_err(|_| actix_web::error::ErrorNotFound("file not found"))?;

    // Render text preview; binary detection is naive
    let is_text = content_bytes.iter().all(|b| b.is_ascii() || *b == b'\n' || *b == b'\r' || *b == b'\t');
    let preview = if is_text {
        let s = String::from_utf8_lossy(&content_bytes);
        html! { pre { code { (s) } } }
    } else {
        html! { p { "Binary file (" (content_bytes.len()) " bytes)" } }
    };

    let content = html! {
        (repo_header(&owner.username, &repo.name, repo.is_private))
        div class="container" {
            div class="main-content" {
                div class="left-content" {
                    div class="content-viewer" {
                        div class="content-header" {
                            span class="content-title" { (format!("📄 {}", blob_path)) }
                        }
                        div class="content-body" {
                            (preview)
                        }
                    }
                }
                div class="sidebar" { }
            }
        }
    };

    Ok(page_shell(&format!("{} / {} - {}", owner.username, repo.name, blob_path), content, user_display.as_deref()))
}