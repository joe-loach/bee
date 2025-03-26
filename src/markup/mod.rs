mod landing;
mod ticket;

pub use ticket::*;

use maud::{html, Markup, DOCTYPE};

use crate::models::user::User;

pub fn root(user: Option<User>) -> Markup {
    html! {
        (head())
        (user_header(user.as_ref()))
        #main-content {
            @if user.is_some() {
                #tickets hx-get="/tickets" hx-trigger="load" { "Loading..." }
            } @else {
                (landing::landing())
            }
        }
    }
}

pub fn head() -> Markup {
    html! {
        (DOCTYPE)
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1";

            title { "Bee Network Tracker" }

            link rel="stylesheet" href="main.css";
            link rel="preconnect" href="https://fonts.googleapis.com";
            link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
            link href="https://fonts.googleapis.com/css2?family=Bricolage+Grotesque:opsz,wght@12..96,200..800&family=Inter:ital,opsz,wght@0,14..32,100..900;1,14..32,100..900&display=swap" rel="stylesheet";
            link rel="stylesheet" href="/fontawesome/css/solid.css";
            link rel="stylesheet" href="/fontawesome/css/fontawesome.css";

            script src="https://unpkg.com/htmx.org@2.0.4" {};
            script src="helpers.js" {};
        }
    }
}

pub fn user_header(user: Option<&User>) -> Markup {
    match user {
        Some(user) => html! {
            header {
                .tabs {
                    .logo hx-get="/" hx-target="body" hx-trigger="click" {
                        img src="/bee.svg" {}
                    }
                    div {
                        h3 {
                            "Logged in as "
                            a .username { (user.username) }
                        }
                    }
                    .spaced {
                        a hx-get="/tickets/add" hx-target="#main-content" { "Add Ticket" }
                        a hx-get="/auth/logout" hx-target="body" { "Logout" }
                    }
                }
            }
        },
        None => html! {
            header {
                .tabs {
                    .logo hx-get="/" hx-target="body" hx-trigger="click" {
                        img src="/bee.svg" {}
                    }
                    .title {
                        h2 { "Bee Network Tracker" }
                    }
                    div {
                        a hx-get="/auth/login" hx-target="#main-content" { "Login" }
                    }
                }
            }
        },
    }
}
