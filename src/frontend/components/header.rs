use maud::{html, Markup};
use crate::frontend::SERVE_PATH;

pub fn header(display_name: Option<&str>) -> Markup {
    html! {
        header {
            div class="header-top" {
                img src=(SERVE_PATH.to_string() + "/gitlit.svg") alt="logo" {}
                h1 { "GitLit" }
                @match display_name {
                    Some(name) => {
                        @let initial = name.chars().next().unwrap_or('?').to_string().to_uppercase();
                        div class="profile" { (initial) }
                        div class="menu" {
                            div class="menu-name" { (name) }
                            div class="menu-options" {
                                a class="menu-text" { "My Repositories" }
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