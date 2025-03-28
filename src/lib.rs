mod auth;
mod database;
mod markup;
mod models;
mod routes;
mod sessions;

use axum::{middleware, routing::get, Extension, Router};
use database::{database, DatabaseConn};
use tower_service::Service;

#[derive(Clone)]
struct State {
    pub db: DatabaseConn,
    pub sessions: sessions::Sessions,
}

fn router(state: State) -> Router {
    Router::new()
        .route("/", get(routes::index))
        .nest("/tickets", routes::ticket::router())
        .nest("/qr", routes::qr::router())
        .nest("/auth", auth::router())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::user_middleware,
        ))
        .layer(Extension(state.clone()))
}

#[worker::event(start)]
fn start() {
    use tracing_subscriber::fmt::format::Pretty;
    use tracing_subscriber::fmt::time::UtcTime;
    use tracing_subscriber::prelude::*;
    use tracing_web::{performance_layer, MakeConsoleWriter};

    let fmt_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_ansi(false) // Only partially supported across JavaScript runtimes
        .with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
        .with_writer(MakeConsoleWriter); // write events to the console
    let perf_layer = performance_layer().with_details_from_fields(Pretty::default());
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init();
}

#[worker::event(fetch)]
async fn fetch(
    req: worker::HttpRequest,
    env: worker::Env,
    _ctx: worker::Context,
) -> worker::Result<axum::http::Response<axum::body::Body>> {
    console_error_panic_hook::set_once();

    let db = database(env.clone());
    let sessions = sessions::sessions(env.clone());

    let state = State {
        db: db.clone(),
        sessions: sessions.clone(),
    };

    let response = router(state).call(req).await?;

    sessions.close().await;
    db.close().await;

    Ok(response)
}
