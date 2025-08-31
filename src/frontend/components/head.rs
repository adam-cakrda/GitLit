use maud::{DOCTYPE, html, Markup};
pub fn head(page_title: &str, css: Markup) -> Markup {
    html! {
        head {
            title { (page_title) }
            meta charset="utf-8" {}
            meta name="viewport" content="width=device-width, initial-scale=1" {}
            meta name="description" content="A simple github alternative written in rust." {}
            link rel="icon" href="gitlit.svg" {}
            link rel="preconnect" href="https://fonts.gstatic.com" {}
            link rel="preconnect" href="https://fonts.googleapis.com" {}
            link rel="stylesheet" href="main.css" {}
            (css)
        }
    }
}