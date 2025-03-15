use maud::{html, Markup};

use crate::models::ticket::{Ticket, TicketDef};

pub fn ticket_area(owned_tickets: &[Ticket], defs: &[TicketDef]) -> Markup {
    html! {
        {
            .ticket-area {
                @for (idx, ticket) in owned_tickets.iter().enumerate() {
                    (ticket_card(ticket, idx))
                }
            }
        }

        @if let Some(form) = ticket_form(owned_tickets, defs) {
            hr;
            (form)
        }
    }
}

fn ticket_form(owned_tickets: &[Ticket], defs: &[TicketDef]) -> Option<Markup> {
    let unclaimed_ticket_defs = defs
        .iter()
        .filter(|t| !owned_tickets.iter().any(|ot| ot.def == t.id))
        .collect::<Vec<_>>();
    if !unclaimed_ticket_defs.is_empty() {
        Some(html! {
            form hx-post="/tickets" hx-target="#tickets" {
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

pub fn ticket_card(ticket: &Ticket, index: usize) -> Markup {
    let expiry_format = time::macros::format_description!(
        "[day padding:none] [month repr:long] [year] at [hour repr:12]:[minute][period case:lower]"
    );

    html! {
        @let query = format!("?ticket={}", ticket.id);

        @let qr = format!("qr_{}", index);
        @let timer = format!("timer_{index}");

        @let try_load = format!("try_load_svg(event,{index})");
        @let save = format!("save_svg(event,{index})");

        @let inc_id = format!("inc_{}", ticket.id);

        @let show_for_time = format!("show_for_time('{qr}','{timer}', '{inc_id}')");

        .ticket-card {
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

            main hx-get={"/qr" (query)} hx-target={"#" (qr)} hx-on::before-request=(try_load) hx-on:click=(show_for_time) {
                .top-box {
                    h3 { (ticket.title) }
                    small { "Bee Network Bus" }
                }
                #(qr) hx-on::after-settle=(save) {}
                div {
                    #(timer) .progress-bar {}
                }
            }

            footer {
                .expiry {
                    p {
                        i .fa-sm .fa-solid .fa-clock style="padding-right: 1em" {}
                        small { "Expires " (ticket.expiry.format(expiry_format).unwrap()) }
                    }
                }

                .statistics {
                    @let usages = format!("usages_{}", ticket.id);
                    @let inc_usage = format!("/tickets/{}/inc", ticket.id);
                    @let dec_usage = format!("/tickets/{}/dec", ticket.id);

                    div {
                        button
                            #(inc_id)
                            .increment
                            hx-post=(inc_usage)
                            hx-target={"#" (usages)}
                            hx-trigger="click, increment"
                        {
                            i .fa-solid .fa-plus {}
                        }
                        div {
                            p #(usages) { (ticket.usages) }
                        }
                        button .decrement hx-post=(dec_usage) hx-target={"#" (usages)} {
                            i .fa-solid .fa-minus {}
                        }
                    }
                }
            }
        }
    }
}
