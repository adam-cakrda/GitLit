use maud::{html, Markup};

pub enum AlertKind {
    Info,
    Warning,
    Error,
    Success,
}

impl AlertKind {
    fn class(&self) -> &'static str {
        match self {
            AlertKind::Info => "alert-info",
            AlertKind::Warning => "alert-warning",
            AlertKind::Error => "alert-error",
            AlertKind::Success => "alert-success",
        }
    }
}

pub fn alert(kind: AlertKind, message: &str) -> Markup {
    html! {
        div class={("alert ".to_string() + kind.class())} {
            span class="alert-icon" { @match kind { AlertKind::Error => { "!" } AlertKind::Warning => { "!" } AlertKind::Success => { "✔" } AlertKind::Info => { "i" } } }
            span class="alert-message" { (message) }
        }
    }
}
