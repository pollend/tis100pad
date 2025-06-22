use std::sync::Arc;

use askama::Template;
use async_stream::try_stream;
use axum::{extract::State, http::request::Parts, response::{Html, IntoResponse}};

use crate::{common::sse::Sse, state::AppState, view::{route_error::{handle_internal_error, RouteError}, sse_events::{SSEEmpty, SSEForm, SSEMergeFragment}, template::{DefaultBase, DefaultTemplate}}};


pub async fn root(
    parts: Parts,
    State(state): State<Arc<AppState>>,
    sse: Option<SSEForm<SSEEmpty>>,
) -> Result<impl IntoResponse, RouteError> {
    #[derive(Template)]
    #[template(path = "pad.html", ext = "html",
        whitespace = "suppress",
        blocks = ["main_content", "main_nav"]
    )]
    struct Tmpl<'a> {
        root: DefaultBase<'a>,
    }
    impl DefaultTemplate for Tmpl<'_> {
        fn __main_content(&self) -> impl askama::FastWritable {
            self.as_main_content()
        }

        fn __main_nav(&self) -> impl askama::FastWritable {
            self.as_main_nav()
        }

        fn __root(&self) -> &DefaultBase<'_> {
            &self.root
        }
    }

    match sse {
        Some(_) => Ok(Sse::<_, RouteError>::new(try_stream! {
            let base = Tmpl {
                root: DefaultBase {
                    title: "Home",
                    base_uri: state.uri.as_str(),
                    manifest: &state.manifest
                }
            };
            yield SSEMergeFragment::new()
                .finish_askam(&base.as_main_content())
                .map_err(handle_internal_error(&state, &parts))?;
        }).into_response()),
        None => {
            let base = Tmpl {
                root: DefaultBase {
                    title: "Home",
                    base_uri: state.uri.as_str(),
                    manifest: &state.manifest,
                }
            };
            Ok(Html(
                base.render()
                    .map_err(handle_internal_error(&state, &parts))?,
            )
            .into_response())
        }
    }
}

