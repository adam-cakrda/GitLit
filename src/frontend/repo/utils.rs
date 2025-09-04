use crate::models::*;
use crate::db::Database;
use crate::frontend::components;
use crate::frontend::SERVE_PATH;
use crate::repo;
use mongodb::bson::doc;
use maud::{Markup, html, PreEscaped, DOCTYPE};
use crate::api::service;

pub fn is_hex_hash(s: &str) -> bool {
    let len = s.len();
    if !(7..=64).contains(&len) { return false; }
    s.chars().all(|c| c.is_ascii_hexdigit())
}
pub async fn token_display(db: &Database, req: &actix_web::HttpRequest) -> Option<String> {
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
pub async fn resolve_owner_repo(
    db: &Database,
    username: &str,
    reponame: &str,
) -> actix_web::Result<(User, Repository)> {
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

pub async fn default_ref(db: &Database, owner: &User, repo: &Repository) -> String {
    match repo::list_branches(&owner._id, &repo._id).await {
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

pub fn entry_icon_and_name(e: &TreeEntry) -> (Markup, Markup) {
    match e.kind {
        EntryKind::Tree => (html! { span class="icon-folder file-icon" {} }, html! { span class="file-name folder" { (&e.path) } }),
        _ => (html! { span class="icon-file file-icon" {} }, html! { span class="file-name" { (&e.path) } }),
    }
}

pub fn render_readme_html(md_bytes: &[u8]) -> Option<Markup> {
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

pub fn join_rel_path(base: Option<&str>, name: &str) -> String {
    match base {
        Some(b) if !b.is_empty() => format!("{}/{}", b.trim_end_matches('/'), name),
        _ => name.to_string(),
    }
}

pub fn breadcrumbs(
    owner_slug: &str,
    repo_slug: &str,
    rev: &str,
    current_path: Option<&str>,
) -> Markup {
    let mut acc: Vec<(String, String)> = Vec::new();
    if let Some(p) = current_path {
        let mut running = String::new();
        for (i, seg) in p.split('/').enumerate() {
            if i == 0 {
                running.push_str(seg);
            } else {
                running.push('/');
                running.push_str(seg);
            }
            acc.push((seg.to_string(), running.clone()));
        }
    }

    html! {
        nav class="breadcrumbs" {
            a href={(format!("/{}/{}/tree/{}", owner_slug, repo_slug, rev))} { (rev) }
            @for (name, partial) in &acc {
                span class="crumb-sep" { "/" }
                a href={(format!("/{}/{}/tree/{}/{}", owner_slug, repo_slug, rev, partial))} { (name) }
            }
        }
    }
}

pub fn parent_path(p: &str) -> Option<String> {
    let mut parts: Vec<&str> = p.split('/').collect();
    if parts.is_empty() { return None; }
    parts.pop();
    if parts.is_empty() { return None; }
    Some(parts.join("/"))
}


pub fn format_time(ts_secs: i64) -> String {
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};
    if let Ok(dt) = OffsetDateTime::from_unix_timestamp(ts_secs) {
        if let Ok(s) = dt.format(&Rfc3339) {
            return s;
        }
    }
    "".to_string()
}

pub fn file_list(
    owner_slug: &str,
    repo_slug: &str,
    rev: &str,
    current_path: Option<&str>,
    entries: &[TreeEntry],
) -> Markup {
    html! {
        ul class="file-list" {
            @for e in entries {
                @let (icon, name_markup) = entry_icon_and_name(e);
                @let rel = join_rel_path(current_path, &e.path);
                @match e.kind {
                    EntryKind::Tree => {
                        li class="file-item" {
                            (icon)
                            a href={(format!("/{}/{}/tree/{}/{}", owner_slug, repo_slug, rev, rel))} { (name_markup) }
                            span class="file-message" { "" }
                            span class="file-time" { "" }
                        }
                    }
                    EntryKind::Blob | EntryKind::Commit | EntryKind::Other(_) => {
                        li class="file-item" {
                            (icon)
                            a href={(format!("/{}/{}/blob/{}/{}", owner_slug, repo_slug, rev, rel))} { (name_markup) }
                            //TODO: last commit and time
                            span class="file-message" { "" }
                            span class="file-time" { "" }
                        }
                    }
                }
            }
        }
    }
}
pub fn page_shell(page_title: &str, inner: Markup, user_display: Option<&str>) -> Markup {
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