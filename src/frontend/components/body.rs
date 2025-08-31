use maud::{DOCTYPE, html, Markup};
use crate::frontend::components;
pub fn body(content: Markup) -> Markup {
    html! {
        body {
            div class="shapes" {
                div class="blob b1" {}
                div class="blob b2" {}
            }

            (components::header())

            main {
                (content)
            }
        }
    }
}