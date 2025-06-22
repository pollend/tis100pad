use std::sync::Arc;

use axum::{routing::get, Router};

use crate::{state::AppState, view};

pub mod sse_events;
pub mod template;
pub mod route_error;
pub mod index;

#[rustfmt::skip]
pub fn view_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(view::index::root))
}


