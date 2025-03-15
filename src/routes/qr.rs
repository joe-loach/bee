use axum::{
    extract::Query,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Extension, Router,
};
use fast_qr::{
    convert::{svg::SvgBuilder, Builder, Shape},
    Mask, QRBuilder, ECL,
};
use serde::Deserialize;

use crate::{
    models::{
        ticket::{self, TicketId},
        user::User,
    },
    State,
};

pub fn router() -> Router {
    Router::new().route("/", get(get_qr_svg))
}

#[derive(Deserialize)]
struct QrData {
    ticket: TicketId,
}

/// Dyanmically creates a QR code in the proper format for the data for the bus app.
async fn get_qr_svg(
    Query(QrData { ticket }): Query<QrData>,
    Extension(user): Extension<Option<User>>,
    Extension(state): Extension<State>,
) -> Response {
    let Some(user) = user else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let Some(ticket) = state.db.query_one(ticket::GetTicket { id: ticket }).await else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    if ticket.user != user.id {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let qrcode = QRBuilder::new(ticket.qr.as_bytes())
        .ecl(ECL::M)
        .mask(Mask::Diamonds)
        .build()
        .unwrap();

    let svg = SvgBuilder::default()
        .margin(0)
        .shape(Shape::Square)
        .to_str(&qrcode);

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/svg+xml")],
        svg.into_bytes(),
    )
        .into_response()
}
