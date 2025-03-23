use maud::{html, Markup};

pub fn landing() -> Markup {
    html! {
        #landing {
            .heading {
                h1 { "Every journey. Every day." }
                div {
                    p {
                        "This is a functional mock-up of the Bee Network App. Built using axum, maud, htmx, deployed to cloudflare workers."
                    }
                    p {
                        "Created by " a href="https://joeloach.co.uk" { "Joe Loach" } ". "
                        "Email me if you want access."
                    }
                }
            }
        }
    }
}