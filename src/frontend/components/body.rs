use maud::{html, Markup};
use crate::frontend::components;

pub fn body(content: Markup, user_display: Option<&str>) -> Markup {
    html! {
        body {
            div class="shapes" {
                div class="blob b1" {}
                div class="blob b2" {}
            }

            (components::header(user_display))

            (content)
        }
    }
}