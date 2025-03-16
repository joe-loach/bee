use axum::{
    extract::Path, http::StatusCode, response::Redirect, routing::{get, post}, Extension, Form, Router
};
use maud::Markup;
use serde::Deserialize;

use crate::{
    markup::{self, ticket_area, ticket_card},
    models::{
        ticket::{self, DefId, Ticket, TicketDef, TicketId, UserTicket},
        user::User,
    },
    State,
};

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_all_tickets))
        .route("/add", get(ticket_form).post(add_ticket))
        .route("/{ticket}", get(get_single_ticket))
        .route("/{ticket}/inc", post(increment_usage))
        .route("/{ticket}/dec", post(decrement_usage))
}

async fn ticket_form(
    Extension(user): Extension<Option<User>>,
    Extension(state): Extension<State>,
) -> Result<Markup, StatusCode> {
    let Some(user) = user else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let defs = state.db.query(ticket::GetAllDefinitions).await;
    let user_tickets = state.db.query(ticket::GetAllFromUser { id: user.id }).await;

    let tickets = tickets_from_defs(user_tickets, &defs);

    markup::ticket_form(&tickets, &defs).ok_or(StatusCode::NO_CONTENT)
}

async fn get_all_tickets(
    Extension(user): Extension<Option<User>>,
    Extension(state): Extension<State>,
) -> Result<Markup, StatusCode> {
    let Some(user) = user else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let defs = state.db.query(ticket::GetAllDefinitions).await;
    let user_tickets = state.db.query(ticket::GetAllFromUser { id: user.id }).await;

    let tickets = tickets_from_defs(user_tickets, &defs);

    Ok(ticket_area(&tickets))
}

#[axum::debug_handler]
async fn increment_usage(
    Path(id): Path<TicketId>,
    Extension(user): Extension<Option<User>>,
    Extension(state): Extension<State>,
) -> Result<String, StatusCode> {
    let Some(user) = user else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let Some(mut user_ticket) = state.db.query_one(ticket::GetTicket { id }).await else {
        return Err(StatusCode::BAD_REQUEST);
    };

    if user_ticket.user != user.id {
        return Err(StatusCode::UNAUTHORIZED);
    }

    if user_ticket.usages == u32::MAX {
        // how on earth did we get here?!
        return Ok(u64::MAX.to_string());
    }

    // increment and update
    user_ticket.usages += 1;
    state
        .db
        .run(ticket::UpdateUsage {
            id,
            usages: user_ticket.usages,
        })
        .await;

    Ok(user_ticket.usages.to_string())
}

async fn decrement_usage(
    Path(id): Path<TicketId>,
    Extension(user): Extension<Option<User>>,
    Extension(state): Extension<State>,
) -> Result<String, StatusCode> {
    let Some(user) = user else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let Some(mut user_ticket) = state.db.query_one(ticket::GetTicket { id }).await else {
        return Err(StatusCode::BAD_REQUEST);
    };

    if user_ticket.user != user.id {
        return Err(StatusCode::UNAUTHORIZED);
    }

    if user_ticket.usages == 0 {
        // can't keep decrementing at 0
        return Ok(0.to_string());
    }

    // decrement and update
    user_ticket.usages -= 1;
    state
        .db
        .run(ticket::UpdateUsage {
            id,
            usages: user_ticket.usages,
        })
        .await;

    Ok(user_ticket.usages.to_string())
}

#[derive(Deserialize)]
struct CreateTicket {
    ticket: DefId,
    qr: String,
}

async fn add_ticket(
    Extension(user): Extension<Option<User>>,
    Extension(state): Extension<State>,
    Form(CreateTicket { ticket, qr }): Form<CreateTicket>,
) -> Result<Redirect, StatusCode> {
    let Some(user) = user else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    state
        .db
        .run(ticket::Insert {
            user: user.id,
            def: ticket,
            qr,
        })
        .await;

    Ok(Redirect::to("/"))
}

async fn get_single_ticket(
    Path(id): Path<TicketId>,
    Extension(user): Extension<Option<User>>,
    Extension(state): Extension<State>,
) -> Result<Markup, StatusCode> {
    let Some(user) = user else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let Some(user_ticket) = state.db.query_one(ticket::GetTicket { id }).await else {
        return Err(StatusCode::BAD_REQUEST);
    };

    if user_ticket.user != user.id {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let defs = state.db.query(ticket::GetAllDefinitions).await;
    let def = defs
        .iter()
        .find(|def| def.id == user_ticket.def)
        .expect("definition exists");

    let ticket = Ticket::combine(user_ticket, def);

    // we dont know the index as we're only fetching one ticket,
    // just give the id of the ticket instead
    // TODO: fix this?
    let index = ticket.id.0 as usize;

    Ok(ticket_card(&ticket, index))
}

fn tickets_from_defs(
    user_tickets: impl IntoIterator<Item = UserTicket>,
    defs: &[TicketDef],
) -> Vec<Ticket> {
    user_tickets
        .into_iter()
        .map(|ut| {
            let ut_def = defs
                .iter()
                .find(|def| def.id == ut.def)
                .expect("definition should exist");
            Ticket::combine(ut, ut_def)
        })
        .collect::<Vec<_>>()
}
