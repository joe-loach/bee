use argon2::{password_hash::SaltString, Argon2};
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    routing::get,
    Extension,
};
use axum::{Form, Router};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use maud::{html, Markup};
use rand_chacha::rand_core::SeedableRng;
use serde::Deserialize;

use crate::{
    models::user::{self, User},
    State,
};

pub fn router() -> Router {
    Router::new()
        .route("/register", get(register_form).post(register))
        .route("/login", get(login_form).post(login))
        .route("/logout", get(logout))
}

async fn login_form() -> Markup {
    html! {
        div style="width: 80%; max-width: 600px; margin: auto; padding: 4em;" {
            form hx-post="/auth/login" hx-target="body" {
                label for="username" {"Username: "}
                input name="username" type="text";

                label for="password" {"Password: "}
                input name="password" type="text";

                input type="submit" value="Login";
            }
        }
    }
}

async fn register_form() -> Markup {
    html! {
        div style="width: 80%; max-width: 600px; margin: auto; padding: 4em;" {
            form hx-post="/auth/register" hx-target="body" {
                label for="username" {"Username: "}
                input name="username" type="text";

                label for="password" {"Password: "}
                input name="password" type="text";

                input type="submit" value="Register";
            }
        }
    }
}

pub async fn user_middleware(
    jar: CookieJar,
    Extension(state): Extension<State>,
    mut request: Request,
    next: Next,
) -> Response {
    if let Some(cookie) = jar.get("session") {
        if let Some(user) = state.sessions.get(cookie.value().to_owned()).await {
            request.extensions_mut().insert(Some(user));
            return next.run(request).await;
        };
    }

    request.extensions_mut().insert(None::<User>);
    next.run(request).await
}

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

pub async fn login(
    jar: CookieJar,
    Extension(state): Extension<State>,
    Form(payload): Form<LoginRequest>,
) -> Response {
    use argon2::PasswordHash;

    let session_id = payload.username.clone();

    let Some(user) = state
        .db
        .query_one(user::Get {
            username: payload.username,
        })
        .await
    else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let hash = PasswordHash::new(&user.password_hash).unwrap();
    if hash
        .verify_password(&[&Argon2::default()], &payload.password)
        .is_err()
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    state.sessions.put(session_id.clone(), user.clone()).await;

    let mut cookie = Cookie::new("session", session_id);
    cookie.set_path("/");

    (jar.add(cookie), crate::markup::root(Some(user))).into_response()
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    username: String,
    password: String,
}

// TODO: check if the username already exists
pub async fn register(
    Extension(state): Extension<State>,
    Form(payload): Form<RegisterRequest>,
) -> impl IntoResponse {
    use argon2::PasswordHasher;

    let salt = SaltString::generate(&mut rand_chacha::ChaCha12Rng::from_seed(Default::default()));

    let argon = Argon2::default();
    let password_hash = argon
        .hash_password(payload.password.as_bytes(), &salt)
        .expect("hashed password");

    state
        .db
        .query(user::Insert {
            username: payload.username,
            password: password_hash.serialize().to_string(),
        })
        .await;

    crate::markup::root(None)
}

pub async fn logout(jar: CookieJar, Extension(state): Extension<State>) -> impl IntoResponse {
    if let Some(cookie) = jar.get("session") {
        state.sessions.remove(cookie.value().to_owned()).await;
    }

    let mut cookie = Cookie::from("session");
    cookie.set_path("/");

    (jar.remove(cookie), crate::markup::root(None))
}
