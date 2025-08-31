use maud::{DOCTYPE, html, Markup};
pub fn header() -> Markup {
    html! {
        header {
            img src="gitlit.svg" alt="logo" {}
            h1 { "GitLit" }
            div class="profile" { "A" }
            dic class="menu" {
                div class="menu-name" { "adam" }
                div class="menu-options" {
                    a class="menu-text" { "My Repositories" }
                    a class="menu-text red" { "Log Out" }
                }
            }
        }
    }
}