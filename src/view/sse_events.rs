use crate::common::sse::{Event, EventBuilder};
use askama::{
    filters::{escape, Text},
    FastWritable, NO_VALUES,
};
use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts},
    http::request::Parts,
};
use bytes::BufMut;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt, time::Duration};
use {
    axum::{
        body::Bytes,
        extract::{FromRequest, OptionalFromRequest, Query, Request},
        http::{self},
        response::{IntoResponse, Response},
    },
    serde::de::DeserializeOwned,
};

pub const RETRY_DURATION: u64 = 1000;

pub trait SSETarget {
    fn sse_target(&self) -> Option<&str>;
}

#[derive(Deserialize, Serialize)]
pub struct SSESelector<'a> {
    pub target: Option<Cow<'a, str>>,
}

impl SSETarget for SSESelector<'_> {
    fn sse_target(&self) -> Option<&str> {
        self.target.as_deref()
    }
}

/// The mode in which a fragment is merged into the DOM.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FragmentMergeMode {
    /// Morphs the fragment into the existing element using idiomorph.
    #[default]
    Morph,
    /// Replaces the inner HTML of the existing element.
    Inner,
    /// Replaces the outer HTML of the existing element.
    Outer,
    /// Prepends the fragment to the existing element.
    Prepend,
    /// Appends the fragment to the existing element.
    Append,
    /// Inserts the fragment before the existing element.
    Before,
    /// Inserts the fragment after the existing element.
    After,
    /// Upserts the attributes of the existing element.
    UpsertAttributes,
}

impl FragmentMergeMode {
    /// Returns the [`FragmentMergeMode`] as a string.
    pub(crate) const fn as_str(&self) -> &str {
        match self {
            Self::Morph => "morph",
            Self::Inner => "inner",
            Self::Outer => "outer",
            Self::Prepend => "prepend",
            Self::Append => "append",
            Self::Before => "before",
            Self::After => "after",
            Self::UpsertAttributes => "upsertAttributes",
        }
    }
}

// append the script to the head of the documetn
pub struct SSEExecuteScriptBuilder<'a> {
    /// `id` can be used by the backend to replay events.
    /// This is part of the SSE spec and is used to tell the browser how to handle the event.
    /// For more details see <https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#id>
    pub id: Option<&'a str>,
    /// `retry` is part of the SSE spec and is used to tell the browser how long to wait before reconnecting if the connection is lost.
    /// For more details see <https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#retry>
    pub retry: Duration,
    /// Whether to remove the script after execution, if not provided the client side will default to `true`.
    pub auto_remove: bool,
    /// A list of attributes to add to the script element, if not provided the Datastar client side will default to `type module`.
    /// Each item in the array ***must*** be a string in the format `key value`.
    pub attributes: &'a [(&'a str, &'a str)],
}
impl<'a> SSEExecuteScriptBuilder<'a> {
    /// Creates a new [`ExecuteScript`] event with the given script.
    pub fn new() -> Self {
        Self {
            id: None,
            retry: Duration::from_millis(RETRY_DURATION),
            auto_remove: true,
            attributes: &[("type", "module")],
        }
    }

    /// Sets the `id` of the [`MergeFragments`] event.
    pub fn id(mut self, id: &'a str) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the `retry` of the [`ExecuteScript`] event.
    pub fn retry(mut self, retry: Duration) -> Self {
        self.retry = retry;
        self
    }

    /// Sets the `script` of the [`ExecuteScript`] event.
    pub fn auto_remove(mut self, auto_remove: bool) -> Self {
        self.auto_remove = auto_remove;
        self
    }

    /// Sets the `attribute` of the [`ExecuteScript`] event.
    pub fn attributes(mut self, attributes: &'a [(&'a str, &'a str)]) -> Self {
        self.attributes = attributes;
        self
    }

    fn finish_header(&self) -> EventBuilder {
        let mut builder = EventBuilder::new();
        builder = builder.event("sse-execute-script");
        if let Some(id) = &self.id {
            builder = builder.id(id);
        }
        builder = builder.retry(self.retry);
        builder = builder.data_nested_field(
            "autoRemove",
            match self.auto_remove {
                true => "true",
                false => "false",
            },
        );
        for attribute in self.attributes {
            builder.0.extend_from_slice("data: attributes ".as_bytes());
            builder.0.extend_from_slice(attribute.0.as_bytes());
            builder.0.put_u8(b' ');
            builder.0.extend_from_slice(attribute.1.as_bytes());
            builder.0.put_u8(b'\n');
        }
        builder
    }

    pub fn finish_redirect(&self, url: &'a str) -> Event {
        let mut builder = self.finish_header();
        let url = escape(url, Text).unwrap();
        builder =
            builder.data_nested_field("script", format!(r#"window.location.href = "{}""#, url));
        builder.finish()
    }

    pub fn finish_title(&self, title: &'a str) -> Event {
        let mut builder = self.finish_header();
        let title = escape(title, Text).unwrap();
        builder = builder.data_nested_field("script", format!(r#"document.title = "{}""#, title));
        builder.finish()
    }

    pub fn finish_history(&self, url: &'a str) -> Event {
        let mut builder = self.finish_header();
        let url = escape(url, Text).unwrap();
        builder = builder.data_nested_field(
            "script",
            format!(r#"window.history.pushState({{}}, "", "{}")"#, url),
        );
        builder.finish()
    }

    pub fn finish_reload(&self) -> Event {
        let mut builder = self.finish_header();
        builder = builder.data_nested_field("script", "window.location.reload()");
        builder.finish()
    }

    pub fn finish(&self, script: &'a str) -> Event {
        let mut builder = self.finish_header();
        for line in script.lines() {
            builder = builder.data_nested_field("script", line);
        }
        builder.finish()
    }
}

pub struct SSEMergeDataAlpineElement<'a> {
    /// `id` can be used by the backend to replay events.
    /// This is part of the SSE spec and is used to tell the browser how to handle the event.
    /// For more details see <https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#id>
    pub id: Option<&'a str>,
    /// `retry` is part of the SSE spec and is used to tell the browser how long to wait before reconnecting if the connection is lost.
    /// For more details see <https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#retry>
    pub retry: Duration,
    /// The CSS selector to exec on fragment.
    /// If not provided, Datastar will default to using the id attribute of the fragment.
    pub selector: Option<&'a str>,
}

impl<'a> SSEMergeDataAlpineElement<'a> {
    pub fn new() -> Self {
        Self {
            id: None,
            retry: Duration::from_millis(RETRY_DURATION),
            selector: None,
        }
    }

    /// Sets the `id` of the [`MergeFragments`] event.
    pub fn id(mut self, id: &'a str) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the `selector` of the [`MergeFragments`] event.
    pub fn selector(mut self, selector: &'a str) -> Self {
        self.selector = Some(selector);
        self
    }

    /// Sets the `retry` of the [`ExecuteScript`] event.
    pub fn retry(mut self, retry: Duration) -> Self {
        self.retry = retry;
        self
    }
    pub fn finish<T>(&self, data: T) -> Result<Event, serde_json::Error>
    where
        T: serde::Serialize,
    {
        let mut builder = EventBuilder::new();
        builder = builder.event("sse-merge-alpine-data");
        if let Some(id) = &self.id {
            builder = builder.id(id);
        }
        if let Some(selector) = self.selector {
            builder = builder.data_nested_field("selector", selector);
        }
        builder = builder.retry(self.retry);
        builder.0.extend_from_slice("data: data ".as_bytes());
        builder = builder.extend_from_json_data(data)?;
        builder.0.put_u8(b'\n');

        Ok(builder.finish())
    }
}

impl<'a> Default for SSEMergeDataAlpineElement<'a> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SSEExecuteOnAlpineElement<'a> {
    /// `id` can be used by the backend to replay events.
    /// This is part of the SSE spec and is used to tell the browser how to handle the event.
    /// For more details see <https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#id>
    pub id: Option<&'a str>,
    /// `retry` is part of the SSE spec and is used to tell the browser how long to wait before reconnecting if the connection is lost.
    /// For more details see <https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#retry>
    pub retry: Duration,
    /// The CSS selector to exec on fragment.
    /// If not provided, Datastar will default to using the id attribute of the fragment.
    pub selector: Option<&'a str>,
}

impl<'a> SSEExecuteOnAlpineElement<'a> {
    pub fn new() -> Self {
        Self {
            id: None,
            retry: Duration::from_millis(RETRY_DURATION),
            selector: None,
        }
    }

    /// Sets the `selector` of the [`MergeFragments`] event.
    pub fn selector(mut self, selector: &'a str) -> Self {
        self.selector = Some(selector);
        self
    }

    /// Sets the `id` of the [`MergeFragments`] event.
    pub fn id(mut self, id: &'a str) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the `retry` of the [`ExecuteScript`] event.
    pub fn retry(mut self, retry: Duration) -> Self {
        self.retry = retry;
        self
    }

    pub fn finish_askam<T>(&self, writer: &T) -> Result<Event, askama::Error>
    where
        T: FastWritable,
    {
        let mut builder = EventBuilder::new();
        builder = builder.event("sse-execute-alpine-data");
        if let Some(id) = &self.id {
            builder = builder.id(id);
        }
        if let Some(selector) = self.selector {
            builder = builder.data_nested_field("selector", selector);
        }
        builder = builder.retry(self.retry);
        struct FragmentWriter<'a> {
            first: bool,
            builder: &'a mut EventBuilder,
        }
        impl<'a> fmt::Write for FragmentWriter<'a> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                if self.first {
                    self.builder.0.extend_from_slice("data: script ".as_bytes());
                    self.first = false;
                }
                let mut last_split = 0;
                for delimiter in memchr::memchr2_iter(b'\n', b'\r', s.as_bytes()) {
                    self.builder
                        .0
                        .extend_from_slice(&s.as_bytes()[last_split..delimiter]);
                    last_split = delimiter + 1;
                    self.builder
                        .0
                        .extend_from_slice("\ndata: script ".as_bytes());
                }
                self.builder
                    .0
                    .extend_from_slice(&s.as_bytes()[last_split..]);
                Ok(())
            }
        }
        let mut f_writer = FragmentWriter {
            first: true,
            builder: &mut builder,
        };
        writer.write_into(&mut f_writer, NO_VALUES)?;
        builder.0.put_u8(b'\n');
        Ok(builder.finish())
    }

    pub fn finish(&self, script: &'a str) -> Event {
        let mut builder = EventBuilder::new();
        builder = builder.event("sse-execute-alpine-data");
        if let Some(id) = &self.id {
            builder = builder.id(id);
        }
        if let Some(selector) = self.selector {
            builder = builder.data_nested_field("selector", selector);
        }
        builder = builder.retry(self.retry);
        for line in script.lines() {
            builder = builder.data_nested_field("script", line);
        }
        builder.finish()
    }
}

pub struct SSERemoveFragment<'a> {
    /// `id` is can be used by the backend to replay events.
    /// This is part of the SSE spec and is used to tell the browser how to handle the event.
    /// For more details see <https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#id>
    pub id: Option<&'a str>,
    /// `retry` is part of the SSE spec and is used to tell the browser how long to wait before reconnecting if the connection is lost.
    /// For more details see <https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#retry>
    pub retry: Duration,
    /// The CSS selector to use to insert the fragments.
    /// If not provided, Datastar will default to using the id attribute of the fragment.
    pub selector: Option<&'a str>,
    /// Whether to use view transitions, if not provided the Datastar client side will default to `false`.
    pub use_view_transition: bool,
}
impl<'a> SSERemoveFragment<'a> {
    /// Creates a new [`MergeFragments`] event with the given fragments.
    pub fn new() -> Self {
        Self {
            id: None,
            retry: Duration::from_millis(RETRY_DURATION),
            selector: None,
            use_view_transition: false,
        }
    }

    /// Sets the `id` of the [`MergeFragments`] event.
    pub fn id(mut self, id: &'a str) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the `retry` of the [`MergeFragments`] event.
    pub fn retry(mut self, retry: Duration) -> Self {
        self.retry = retry;
        self
    }

    /// Sets the `selector` of the [`MergeFragments`] event.
    pub fn selector(mut self, selector: &'a str) -> Self {
        self.selector = Some(selector);
        self
    }
    /// Sets the `use_view_transition` of the [`MergeFragments`] event.
    pub fn use_view_transition(mut self, use_view_transition: bool) -> Self {
        self.use_view_transition = use_view_transition;
        self
    }

    pub fn finish(&self) -> Event {
        let mut builder = EventBuilder::new();
        builder = builder.event("sse-remove-fragments");
        if let Some(id) = &self.id {
            builder = builder.id(id);
        }
        builder = builder.retry(self.retry);
        if let Some(selector) = self.selector {
            builder = builder.data_nested_field("selector", selector);
        }
        builder = builder.data_nested_field(
            "useViewTransition",
            match self.use_view_transition {
                true => "true",
                false => "false",
            },
        );
        builder.finish()
    }
}

pub struct SSEError<'a> {
    error: &'a str,

    // consume the error will prevent the client from retrying the connection
    consume: bool,
}

impl<'a> SSEError<'a> {
    /// Creates a new [`SSEError`] event with the given error message.
    pub fn new(error: &'a str) -> Self {
        Self {
            error,
            consume: false,
        }
    }

    /// Sets whether the error should consume the connection.
    pub fn consume(mut self, consume: bool) -> Self {
        self.consume = consume;
        self
    }

    pub fn finish(&self) -> Event {
        let mut builder = EventBuilder::new();
        builder = builder.event("sse-error");
        builder = builder.data_nested_field("error", self.error);
        builder = builder.data_nested_field(
            "consume",
            match self.consume {
                true => "true",
                false => "false",
            },
        );
        builder.finish()
    }
}

pub struct SSEMergeFragment<'a> {
    /// `id` is can be used by the backend to replay events.
    /// This is part of the SSE spec and is used to tell the browser how to handle the event.
    /// For more details see <https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#id>
    pub id: Option<&'a str>,
    /// `retry` is part of the SSE spec and is used to tell the browser how long to wait before reconnecting if the connection is lost.
    /// For more details see <https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#retry>
    pub retry: Duration,
    /// The CSS selector to use to insert the fragments.
    /// If not provided, Datastar will default to using the id attribute of the fragment.
    pub selector: Option<&'a str>,
    /// The mode to use when merging the fragment into the DOM.
    /// If not provided the Datastar client side will default to [`FragmentMergeMode::Morph`].
    pub merge_mode: FragmentMergeMode,
    /// Whether to use view transitions, if not provided the Datastar client side will default to `false`.
    pub use_view_transition: bool,
}

impl<'a> SSEMergeFragment<'a> {
    /// Creates a new [`MergeFragments`] event with the given fragments.
    pub fn new() -> Self {
        Self {
            id: None,
            retry: Duration::from_millis(RETRY_DURATION),
            selector: None,
            merge_mode: FragmentMergeMode::default(),
            use_view_transition: false,
        }
    }

    /// Sets the `id` of the [`MergeFragments`] event.
    pub fn id(mut self, id: &'a str) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the `retry` of the [`MergeFragments`] event.
    pub fn retry(mut self, retry: Duration) -> Self {
        self.retry = retry;
        self
    }

    /// Sets the `selector` of the [`MergeFragments`] event.
    pub fn selector(mut self, selector: &'a str) -> Self {
        self.selector = Some(selector);
        self
    }

    /// Sets the `merge_mode` of the [`MergeFragments`] event.
    pub fn merge_mode(mut self, merge_mode: FragmentMergeMode) -> Self {
        self.merge_mode = merge_mode;
        self
    }

    /// Sets the `use_view_transition` of the [`MergeFragments`] event.
    pub fn use_view_transition(mut self, use_view_transition: bool) -> Self {
        self.use_view_transition = use_view_transition;
        self
    }
    fn finish_header(&self) -> EventBuilder {
        let mut builder = EventBuilder::new();
        builder = builder.event("sse-merge-fragments");
        if let Some(id) = &self.id {
            builder = builder.id(id);
        }
        builder = builder.retry(self.retry);
        if let Some(selector) = self.selector {
            builder = builder.data_nested_field("selector", selector);
        }
        if self.merge_mode != FragmentMergeMode::default() {
            builder = builder.data_nested_field("mergeMode", self.merge_mode.as_str());
        }
        builder = builder.data_nested_field(
            "useViewTransition",
            match self.use_view_transition {
                true => "true",
                false => "false",
            },
        );
        builder
    }

    pub fn finish(&self, fragments: &'a str) -> Event {
        let mut builder = self.finish_header();
        for line in fragments.lines() {
            builder = builder.data_nested_field("fragments", line);
        }
        builder.finish()
    }

    pub fn finish_askam<T>(&self, writer: &T) -> Result<Event, askama::Error>
    where
        T: FastWritable,
    {
        let mut builder = self.finish_header();
        struct FragmentWriter<'a> {
            first: bool,
            builder: &'a mut EventBuilder,
        }
        impl<'a> fmt::Write for FragmentWriter<'a> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                if self.first {
                    self.builder
                        .0
                        .extend_from_slice("data: fragments ".as_bytes());
                    self.first = false;
                }
                let mut last_split = 0;
                for delimiter in memchr::memchr2_iter(b'\n', b'\r', s.as_bytes()) {
                    self.builder
                        .0
                        .extend_from_slice(&s.as_bytes()[last_split..delimiter]);
                    last_split = delimiter + 1;
                    self.builder
                        .0
                        .extend_from_slice("\ndata: fragments ".as_bytes());
                }
                self.builder
                    .0
                    .extend_from_slice(&s.as_bytes()[last_split..]);
                Ok(())
            }
        }
        let mut f_writer = FragmentWriter {
            first: true,
            builder: &mut builder,
        };
        writer.write_into(&mut f_writer, NO_VALUES)?;
        builder.0.put_u8(b'\n');
        Ok(builder.finish())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SSEEmpty {}

#[derive(Deserialize)]
struct SSEParam {
    sse: serde_json::Value,
}
/// [`ReadSignals`] is a request extractor that reads datastar signals from the request.
///
/// # Examples
///
/// ```
/// use datastar::prelude::ReadSignals;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Signals {
///     foo: String,
///     bar: i32,
/// }
///
/// async fn handler(ReadSignals(signals): ReadSignals<Signals>) {
///    println!("foo: {}", signals.foo);
///    println!("bar: {}", signals.bar);
/// }
///
/// ```
#[derive(Debug)]
pub struct SSEForm<T: DeserializeOwned>(pub T);

pub fn is_sse_request(req: &Parts) -> bool {
    req.headers.get("sse-request").is_some()
}

impl<T: DeserializeOwned, S: Send + Sync> OptionalFromRequest<S> for SSEForm<T>
where
    Bytes: FromRequest<S>,
{
    type Rejection = Response;
    async fn from_request(req: Request, state: &S) -> Result<Option<Self>, Self::Rejection> {
        if let None = req.headers().get("sse-request") {
            return Ok(None);
        }
        let result = <Self as FromRequest<S>>::from_request(req, state).await;
        match result {
            Ok(v) => Ok(Some(v)),
            Err(e) => Err(e),
        }
    }
}

impl<T: DeserializeOwned, S: Send + Sync> FromRequest<S> for SSEForm<T>
where
    Bytes: FromRequest<S>,
{
    type Rejection = Response;
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let json = match *req.method() {
            http::Method::GET => {
                let query = Query::<SSEParam>::from_request(req, state)
                    .await
                    .map_err(IntoResponse::into_response)?;

                let signals = query.0.sse.as_str().ok_or(
                    (http::StatusCode::BAD_REQUEST, "Failed to parse JSON").into_response(),
                )?;

                serde_json::from_str(signals)
            }
            _ => {
                let body = Bytes::from_request(req, state)
                    .await
                    .map_err(IntoResponse::into_response)?;

                serde_json::from_slice(&body)
            }
        }
        .map_err(|_| (http::StatusCode::BAD_REQUEST, "Failed to parse JSON").into_response())?;

        Ok(Self(json))
    }
}

impl<T: DeserializeOwned, S: Send + Sync> OptionalFromRequestParts<S> for SSEForm<T>
where
    Bytes: FromRequestParts<S>,
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        if let None = parts.headers.get("sse-request") {
            return Ok(None);
        }
        let result = <Self as FromRequestParts<S>>::from_request_parts(parts, state).await;
        match result {
            Ok(v) => Ok(Some(v)),
            Err(e) => Err(e),
        }
    }
}

impl<T: DeserializeOwned, S: Send + Sync> FromRequestParts<S> for SSEForm<T>
where
    Bytes: FromRequestParts<S>,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let json = match parts.method {
            http::Method::GET => {
                let query = Query::<SSEParam>::from_request_parts(parts, state)
                    .await
                    .map_err(IntoResponse::into_response)?;

                let signals = query.0.sse.as_str().ok_or(
                    (http::StatusCode::BAD_REQUEST, "Failed to parse JSON").into_response(),
                )?;

                serde_json::from_str(signals)
            }
            _ => {
                let body = Bytes::from_request_parts(parts, state)
                    .await
                    .map_err(IntoResponse::into_response)?;

                serde_json::from_slice(&body)
            }
        }
        .map_err(|_| (http::StatusCode::BAD_REQUEST, "Failed to parse JSON").into_response())?;

        Ok(Self(json))
    }
}

