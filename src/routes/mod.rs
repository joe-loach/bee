pub mod qr;
pub mod ticket;

use axum::Extension;
use maud::Markup;

use crate::{markup, models::user::User};

pub async fn index(Extension(user): Extension<Option<User>>) -> Markup {
    markup::root(user)
}
