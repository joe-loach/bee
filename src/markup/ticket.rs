use maud::{html, Markup};
use time::UtcDateTime;

use crate::models::ticket::{Ticket, TicketDef};

pub fn ticket_area(owned_tickets: &[Ticket]) -> Markup {
    html! {
        {
            #ticket-area {
                @for ticket in owned_tickets.iter() {
                    (ticket_card(TicketMarkup::Small { ticket }))
                }
            }
        }
    }
}

pub fn ticket_form(owned_tickets: &[Ticket], defs: &[TicketDef]) -> Option<Markup> {
    let unclaimed_ticket_defs = defs
        .iter()
        .filter(|t| !owned_tickets.iter().any(|ot| ot.def == t.id))
        .collect::<Vec<_>>();
    if !unclaimed_ticket_defs.is_empty() {
        Some(html! {
            form hx-post="/tickets/add" hx-target="body" {
                label for="ticket" {"Ticket: "}
                select id="ticket" name="ticket" {
                    @for def in unclaimed_ticket_defs {
                        option value=(def.id) { (def.title) }
                    }
                }

                br;

                label for="qr" {"QR Data "}
                input name="qr" type="text" placeholder="7,1,47,128c93669b...";

                br;

                input type="submit" value="Add";
            }
        })
    } else {
        None
    }
}

pub enum TicketMarkup<'t> {
    Large { ticket: &'t Ticket },
    Small { ticket: &'t Ticket },
}

pub fn ticket_card(markup: TicketMarkup) -> Markup {
    match markup {
        TicketMarkup::Large { ticket } => large(ticket),
        TicketMarkup::Small { ticket } => small(ticket),
    }
}

fn large(ticket: &Ticket) -> Markup {
    let ticket_qr = format!("/qr?ticket={}", ticket.id);
    let increment = format!("increment({})", ticket.id);

    html! {
        @let save_svg = format!("save_svg(event, {})", 1);

        .large-ticket {
            header {
                h3 { "Your Ticket"}
                button .close-button hx-get="/tickets" hx-target="#tickets" hx-on::before-send=(increment) { "Close" }
            }
            .ticket-card style="margin-top: 1em; margin-bottom: 1em" {
                header {
                    div {
                        i .fa-sm .fa-solid .fa-bus-simple {}
                        small style="padding-inline: 0.5em;" { "Bus" }
                    }
                    div {
                        i .fa-sm .fa-solid .fa-user {}
                        small style="padding-inline: 0.5em;" { "Student" }
                    }
                }
                main {
                    #qr hx-get=(ticket_qr) hx-trigger="load" hx-on::after-settle=(save_svg) {}
                    (expiry(&ticket.expiry, true))
                    .moving-bee {
                        hr;
                        .bee-container {
                            .bee {}
                        }
                    }
                    div {
                        h3 { (ticket.title) }
                        small .sub { "Students must show valid ID on use" }
                        div style="line-height: 1.5" {
                            p {
                                i .fa-sm .fa-solid .fa-circle-check .fa-fw style="padding-right: 0.5em" {}
                                small { "Bee Network" }
                            }
                            p {
                                i .fa-sm .fa-solid .fa-lock .fa-fw style="padding-right: 0.5em" {}
                                small { "Ticket locked to this device" }
                            }
                        }
                    }
                }
                div style="padding-top: 1.5em" { hr; }
                footer {
                    div {
                        p {
                            i .fa-sm .fa-solid .fa-circle-info .fa-fw style="padding-right: 0.5em" {}
                            small { "View details" }
                        }
                    }
                }
            }
        }
    }
}

fn small(ticket: &Ticket) -> Markup {
    let large_ticket = format!("/tickets/{}", ticket.id);

    html! {
        .ticket-card hx-get=(large_ticket) hx-trigger="click" hx-target="#ticket-area" {
            header {
                div {
                    i .fa-sm .fa-solid .fa-bus-simple {}
                    small style="padding-inline: 0.5em;" { "Bus" }
                }
                div {
                    i .fa-sm .fa-solid .fa-user {}
                    small style="padding-inline: 0.5em;" { "Student" }
                }
            }
            main {
                div {
                    h3 { (ticket.title) }
                    small .sub { "Bee Network Bus" }
                }
                div style="width: 100%; margin-inline: 0" { hr; }
                (expiry(&ticket.expiry, false))
            }
            footer {
                .statistics {
                    p { (ticket.usages) " Uses" }
                }
            }
        }
    }
}

fn expiry(expiry: &UtcDateTime, fullscreen: bool) -> Markup {
    let expiry = {
        let expiry_format = time::macros::format_description!(
            "[day padding:none] [month repr:long] [year] at [hour repr:12 padding:none]:[minute][period case:lower]"
        );
        expiry.format(expiry_format).unwrap()
    };

    let style = fullscreen.then_some("padding-top: 1em; padding-bottom: 1em;");

    html! {
        .expiry {
            p style=[style] {
                i .fa-sm .fa-solid .fa-clock style="padding-right: 1em" {}
                small { "Expires " (expiry) }
            }
        }
    }
}
