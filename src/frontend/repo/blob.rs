use crate::models::*;
use actix_web::{Result, web, HttpRequest, get};
use crate::db::Database;
use maud::{Markup, html};
use crate::frontend::components;
use crate::frontend::SERVE_PATH;
use crate::repo;
use crate::frontend::repo::utils;

#[get("/{username}/{reponame}/blob/{rev}/{path:.*}")]
pub async fn blob(
    db: web::Data<Database>,
    req: HttpRequest,
    path: web::Path<(String, String, String, String)>,
) -> Result<Markup> {
    let (username, reponame, rev_in, blob_path) = path.into_inner();
    let (owner, repo) = utils::resolve_owner_repo(&db, &username, &reponame).await?;
    let user_display = utils::token_display(&db, &req).await;

    let (rev, branch_opt_owned): (String, Option<String>) = if utils::is_hex_hash(&rev_in) {
        (rev_in.clone(), None)
    } else {
        (rev_in.clone(), Some(rev_in))
    };
    let branch_opt = branch_opt_owned.as_deref();
    let reference = branch_opt.unwrap_or(&rev);

    let content_bytes = repo::get_file_content(&owner._id, &repo._id, &rev, branch_opt, &blob_path)
        .await
        .map_err(|_| actix_web::error::ErrorNotFound("file not found"))?;

    let is_text = content_bytes.iter().all(|b| b.is_ascii() || *b == b'\n' || *b == b'\r' || *b == b'\t');
    let preview = if is_text {
        let s = String::from_utf8_lossy(&content_bytes);
        html! { pre { code { (s) } } }
    } else {
        html! { p { "Binary file (" (content_bytes.len()) " bytes)" } }
    };

    let parent_path = utils::parent_path(&blob_path);

    let content = html! {
        (components::repo_header(&owner.username, &repo.name, repo.is_private))
        div class="container" {
            div class="main-content" {
                div class="left-content" {
                    (utils::breadcrumbs(&owner.username, &repo.name, reference, parent_path.as_deref()))
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

    Ok(utils::page_shell(&format!("{} / {} - {}", owner.username, repo.name, blob_path), content, user_display.as_deref()))
}