use serde::{Deserialize, Serialize};
use time::UtcDateTime;

use crate::{database, models::user::UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DefId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TicketId(pub u64);

#[derive(Clone, Serialize, Deserialize)]
pub struct TicketDef {
    pub id: DefId,
    pub title: String,
    pub price: u64,
    pub start: String,
    pub expiry: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserTicket {
    pub id: TicketId,
    pub def: DefId,
    pub user: UserId,
    pub qr: String,
    pub usages: u64,
}

#[allow(unused)]
pub struct Ticket {
    pub id: TicketId,
    pub def: DefId,
    pub title: String,
    pub price: u64,
    pub start: UtcDateTime,
    pub expiry: UtcDateTime,
    pub qr: String,
    pub usages: u64,
}

impl Ticket {
    pub fn combine(user_ticket: UserTicket, def: &TicketDef) -> Self {
        assert_eq!(
            user_ticket.def, def.id,
            "used the wrong definition to create ticket"
        );

        let start_time = UtcDateTime::parse(
            &def.start,
            &time::format_description::well_known::Iso8601::DEFAULT,
        )
        .expect("start should be in the correct format in DB");

        let expiry_time = UtcDateTime::parse(
            &def.expiry,
            &time::format_description::well_known::Iso8601::DEFAULT,
        )
        .expect("expiry should be in the correct format in DB");

        Self {
            id: user_ticket.id,
            def: def.id,
            title: def.title.clone(),
            price: def.price,
            start: start_time,
            expiry: expiry_time,
            qr: user_ticket.qr,
            usages: user_ticket.usages,
        }
    }
}

pub struct GetTicket {
    pub id: TicketId,
}

impl database::Query for GetTicket {
    type Result = UserTicket;

    fn query(&self) -> &'static str {
        "SELECT * from user_tickets WHERE id = ?1"
    }

    fn bindings(&self) -> Vec<database::Binding> {
        vec![(self.id.0 as f64).into()]
    }
}

pub struct GetAllFromUser {
    pub id: UserId,
}

impl database::Query for GetAllFromUser {
    type Result = UserTicket;

    fn query(&self) -> &'static str {
        "SELECT * FROM user_tickets WHERE user = ?1"
    }

    fn bindings(&self) -> Vec<database::Binding> {
        vec![(self.id.0 as f64).into()]
    }
}

pub struct UpdateUsage {
    pub id: TicketId,
    pub usages: u64,
}

impl database::Query for UpdateUsage {
    type Result = ();

    fn query(&self) -> &'static str {
        "UPDATE user_tickets SET usages = ?1 WHERE id = ?2"
    }

    fn bindings(&self) -> Vec<js_sys::wasm_bindgen::JsValue> {
        vec![(self.usages as f64).into(), (self.id.0 as f64).into()]
    }
}

pub struct Insert {
    pub user: UserId,
    pub def: DefId,
    pub qr: String,
}

impl database::Query for Insert {
    type Result = ();

    fn query(&self) -> &'static str {
        "INSERT INTO user_tickets (user, def, qr) VALUES (?1, ?2, ?3)"
    }

    fn bindings(&self) -> Vec<js_sys::wasm_bindgen::JsValue> {
        vec![
            (self.user.0 as f64).into(),
            (self.def.0 as f64).into(),
            self.qr.as_str().into(),
        ]
    }
}

pub struct GetAllDefinitions;

impl database::Query for GetAllDefinitions {
    type Result = TicketDef;

    fn query(&self) -> &'static str {
        "SELECT * FROM ticket_defs"
    }

    fn bindings(&self) -> Vec<js_sys::wasm_bindgen::JsValue> {
        vec![]
    }
}

impl std::fmt::Display for DefId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Display for TicketId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
