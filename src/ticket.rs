use serde::{Deserialize, Serialize};
use time::UtcDateTime;

use crate::user::UserId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DefId(pub u64);

impl std::fmt::Display for DefId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TicketId(pub u64);

impl std::fmt::Display for TicketId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

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

pub fn tickets_from_defs(
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
