use crate::{
    common::sse::{EmptySSEEvent, IntoSSEEvent, Sse},
    state::AppState,
};
use async_stream::try_stream;
use axum::{
    http::request::Parts,
    response::{IntoResponse, Redirect},
};
use std::{ops::FnOnce, sync::Arc};
use tracing::error;

use super::sse_events::{is_sse_request, SSEError};

pub enum Reason {
    Internal,
    NotFound,
    NotAuthenticated, // user is un autenticated on a route that requires authentication
}

pub struct RouteError {
    part: Parts,
    reason: Reason,
    state: Arc<AppState>,
}

pub fn handle_internal_error<'a, E>(
    state: &'a Arc<AppState>,
    parts: &'a Parts,
) -> impl FnOnce(E) -> RouteError + 'a
where
    E: std::error::Error + Send + Sync + 'static,
{
    move |e: E| {
        error!("{}", e);
        RouteError::new(parts, state).reason(Reason::Internal)
    }
}

impl RouteError {
    pub fn new(parts: &Parts, state: &Arc<AppState>) -> Self {
        Self {
            part: parts.clone(),
            reason: Reason::Internal,
            state: state.clone(),
        }
    }

    pub fn reason(mut self, reason: Reason) -> Self {
        self.reason = reason;
        self
    }

    pub fn parts(&self) -> &Parts {
        &self.part
    }
}

impl IntoResponse for RouteError {
    fn into_response(self) -> axum::response::Response {
        if is_sse_request(&self.part) {
            return Sse::<_, EmptySSEEvent>::new(try_stream! {
                yield self.into_event()
            })
            .into_response();
        }
        match self.reason {
            Reason::Internal => {
                // Handle internal error
                axum::response::Response::builder()
                    .status(500)
                    .body("Internal Server Error".into())
                    .unwrap()
            }
            Reason::NotFound => {
                // Handle not found error
                axum::response::Response::builder()
                    .status(404)
                    .body("Not Found".into())
                    .unwrap()
            }
            Reason::NotAuthenticated => Redirect::to("/login").into_response(),
        }
    }
}

impl IntoSSEEvent for RouteError {
    fn into_event(self) -> crate::common::sse::Event {
        match self.reason {
            Reason::Internal => SSEError::new("internal").finish(),
            Reason::NotFound => SSEError::new("not_found").consume(true).finish(),
            Reason::NotAuthenticated => todo!(),
        }
    }
}

