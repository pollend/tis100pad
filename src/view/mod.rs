use std::sync::Arc;

use axum::{routing::get, Router};

use crate::{common::unq_id::UnqID, state::AppState, view};

pub mod sse_events;
pub mod template;
pub mod route_error;
pub mod index;
pub mod pad;

pub const MAIN_ROOT_UID: UnqID = UnqID::new_str("main-root");

#[rustfmt::skip]
pub fn view_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(view::index::root))
        .route("/pad", get(view::pad::root))
}


