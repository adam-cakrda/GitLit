use maud::{html, Markup};
use crate::frontend::SERVE_PATH;

pub fn header(display_name: Option<&str>) -> Markup {
    html! {
        header {
            div class="header-top" {
                a href="/" {
                    img src=(SERVE_PATH.to_string() + "/gitlit.svg") alt="logo" {}
                }
                a href="/" {
                    h1 { "GitLit" }
                }
                @match display_name {
                    Some(name) => {
                        @let initial = name.chars().next().unwrap_or('?').to_string().to_uppercase();
                        a href="/new" class="create-repo-btn" {
                            img src=(SERVE_PATH.to_string() + "/create.svg") alt="Create repository" {}
                        }
                        div class="profile" { (initial) }
                        div class="menu" {
                            div class="menu-name" { (name) }
                            div class="menu-options" {
                                a href=("/".to_string() + name) class="menu-text" { "My Repositories" }
                                form method="post" class="menu-text" action="/logout" {
                                    button type="submit" class="red" { "Log Out" }
                                }
                            }
                        }
                    }
                    None => {
                        div class="auth-buttons" {
                            a href="/login" class="auth-btn-secondary" { "Log in" }
                            a href="/register" class="auth-btn-primary" { "Sign up" }
                        }
                    }
                }
            }
        }
    }
}

pub fn repo_header(
    display_name: Option<&str>,
    owner_slug: &str,
    repo_slug: &str,
    is_private: bool,
) -> Markup {
    let visibility = if is_private { "Private" } else { "Public" };
    html! {
        header {
            div class="header-top" {
                a href="/" {
                    img src=(SERVE_PATH.to_string() + "/gitlit.svg") alt="logo" {}
                }
                div class="name" {
                    a href={(format!("/{}", owner_slug))} { (owner_slug) }
                    " / "
                    a href={(format!("/{}/{}", owner_slug, repo_slug))} class="white" { (repo_slug) }
                    div class="repo-visibility" { (visibility) }
                }
                @match display_name {
                    Some(name) => {
                        @let initial = name.chars().next().unwrap_or('?').to_string().to_uppercase();
                        a href="/new" class="auth-btn-secondary create-repo-btn" {
                            img src=(SERVE_PATH.to_string() + "/create.svg") alt="Create repository" {}
                        }
                        div class="profile" { (initial) }
                        div class="menu" {
                            div class="menu-name" { (name) }
                            div class="menu-options" {
                                a href=("/".to_string() + name) class="menu-text" { "My Repositories" }
                                form method="post" class="menu-text" action="/logout" {
                                    button type="submit" class="red" { "Log Out" }
                                }
                            }
                        }
                    }
                    None => {
                        div class="auth-buttons" {
                            a href="/login" class="auth-btn-secondary" { "Log in" }
                            a href="/register" class="auth-btn-primary" { "Sign up" }
                        }
                    }
                }
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