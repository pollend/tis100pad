use std::{
    convert::Infallible,
    fmt::{self},
    future::Future,
    pin::Pin,
    task::{ready, Context, Poll},
    time::Duration,
};

use axum::{
    body::{Body, HttpBody},
    http::{self},
    response::{IntoResponse, Response},
};
use bytes::{BufMut, Bytes, BytesMut};
use futures::{Stream, TryStream};
use http_body::Frame;
use pin_project_lite::pin_project;
use sync_wrapper::SyncWrapper;
use tokio::time::Sleep;

/// An SSE response
#[derive(Clone)]
#[must_use]
pub struct Sse<S, E> {
    _phantom: std::marker::PhantomData<E>,
    stream: S,
    keep_alive: Option<KeepAlive>,
}

pub trait IntoSSEEvent {
    #[must_use]
    fn into_event(self) -> Event;
}

pub struct EmptySSEEvent;
impl IntoSSEEvent for EmptySSEEvent {
    fn into_event(self) -> Event {
        Event(Bytes::new())
    }
}

impl<S, E> Sse<S, E> {
    /// Create a new [`Sse`] response that will respond with the given stream of
    /// [`Event`]s.
    ///
    /// See the [module docs](self) for more details.
    pub fn new(stream: S) -> Self
    where
        S: TryStream<Ok = Event, Error = E> + Send + 'static,
        E: IntoSSEEvent,
    {
        Sse {
            stream,
            keep_alive: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Configure the interval between keep-alive messages.
    ///
    /// Defaults to no keep-alive messages.
    pub fn keep_alive(mut self, keep_alive: KeepAlive) -> Self {
        self.keep_alive = Some(keep_alive);
        self
    }
}

impl<S, E> fmt::Debug for Sse<S, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sse")
            .field("stream", &format_args!("{}", std::any::type_name::<S>()))
            .field("keep_alive", &self.keep_alive)
            .finish()
    }
}

impl<S, E> IntoResponse for Sse<S, E>
where
    S: Stream<Item = Result<Event, E>> + Send + 'static,
    E: IntoSSEEvent,
{
    fn into_response(self) -> Response {
        (
            [
                (http::header::CONTENT_TYPE, mime::TEXT_EVENT_STREAM.as_ref()),
                (http::header::CACHE_CONTROL, "no-cache"),
                (http::header::CONNECTION, "keep-alive"),
            ],
            Body::new(SseBody {
                event_stream: SyncWrapper::new(self.stream),
                keep_alive: self.keep_alive.map(KeepAliveStream::new),
            }),
        )
            .into_response()
    }
}

pin_project! {
    struct SseBody<S> {
        #[pin]
        event_stream: SyncWrapper<S>,
        #[pin]
        keep_alive: Option<KeepAliveStream>,
    }
}

impl<S, E> HttpBody for SseBody<S>
where
    S: Stream<Item = Result<Event, E>>,
    E: IntoSSEEvent,
{
    type Data = Bytes;
    type Error = Infallible;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let this = self.project();
        match this.event_stream.get_pin_mut().poll_next(cx) {
            Poll::Pending => {
                if let Some(keep_alive) = this.keep_alive.as_pin_mut() {
                    keep_alive.poll_event(cx).map(|e| Some(Ok(Frame::data(e))))
                } else {
                    Poll::Pending
                }
            }
            Poll::Ready(Some(Ok(event))) => {
                if let Some(keep_alive) = this.keep_alive.as_pin_mut() {
                    keep_alive.reset();
                }
                Poll::Ready(Some(Ok(Frame::data(event.0))))
            }
            Poll::Ready(Some(Err(error))) => {
                Poll::Ready(Some(Ok(Frame::data(error.into_event().0))))
            }
            Poll::Ready(None) => Poll::Ready(None),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct EventBuilder(pub BytesMut);

impl EventBuilder {
    pub fn new() -> Self {
        EventBuilder(BytesMut::new())
    }

    // custom extension custom space seperated field
    pub fn data_nested_field(mut self, name: &str, value: impl AsRef<[u8]>) -> Self {
        let value = value.as_ref();
        assert_eq!(
            memchr::memchr2(b'\r', b'\n', value),
            None,
            "SSE field value cannot contain newlines or carriage returns",
        );
        self.0.extend_from_slice("data: ".as_bytes());
        self.0.extend_from_slice(name.as_bytes());
        self.0.put_u8(b' ');
        self.0.extend_from_slice(value);
        self.0.put_u8(b'\n');
        self
    }

    pub fn field(mut self, name: &str, value: impl AsRef<[u8]>) -> Self {
        let value = value.as_ref();
        assert_eq!(
            memchr::memchr2(b'\r', b'\n', value),
            None,
            "SSE field value cannot contain newlines or carriage returns",
        );
        self.0.extend_from_slice(name.as_bytes());
        self.0.put_u8(b':');
        self.0.put_u8(b' ');
        self.0.extend_from_slice(value);
        self.0.put_u8(b'\n');
        self
    }

    pub fn extend_from_json_data<T>(mut self, data: T) -> Result<EventBuilder, serde_json::Error>
    where
        T: serde::Serialize,
    {
        struct IgnoreNewLines<'a>(bytes::buf::Writer<&'a mut BytesMut>);
        impl std::io::Write for IgnoreNewLines<'_> {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                let mut last_split = 0;
                for delimiter in memchr::memchr2_iter(b'\n', b'\r', buf) {
                    self.0.write_all(&buf[last_split..delimiter])?;
                    last_split = delimiter + 1;
                }
                self.0.write_all(&buf[last_split..])?;
                Ok(buf.len())
            }

            fn flush(&mut self) -> std::io::Result<()> {
                self.0.flush()
            }
        }
        serde_json::to_writer(IgnoreNewLines((&mut self.0).writer()), &data)?;
        Ok(self)
    }

    pub fn event<T>(self, event: T) -> Self
    where
        T: AsRef<str>,
    {
        self.field("event", event.as_ref().as_bytes())
    }

    pub fn id<T>(self, id: T) -> Self
    where
        T: AsRef<str>,
    {
        let id = id.as_ref().as_bytes();
        assert_eq!(
            memchr::memchr(b'\0', id),
            None,
            "Event ID cannot contain null characters",
        );
        self.field("id", id)
    }

    pub fn json_data<T>(mut self, data: T) -> Result<EventBuilder, serde_json::Error>
    where
        T: serde::Serialize,
    {
        self.0.extend_from_slice(b"data: ");
        self = self.extend_from_json_data(data)?;
        self.0.put_u8(b'\n');
        Ok(self)
    }

    /// Set the event's retry timeout field (`retry:<timeout>`).
    ///
    /// This sets how long clients will wait before reconnecting if they are disconnected from the
    /// SSE endpoint. Note that this is just a hint: clients are free to wait for longer if they
    /// wish, such as if they implement exponential backoff.
    pub fn retry(mut self, duration: Duration) -> EventBuilder {
        self.0.extend_from_slice(b"retry: ");

        let secs = duration.as_secs();
        let millis = duration.subsec_millis();

        if secs > 0 {
            // format seconds
            self.0
                .extend_from_slice(itoa::Buffer::new().format(secs).as_bytes());

            // pad milliseconds
            if millis < 10 {
                self.0.extend_from_slice(b"00");
            } else if millis < 100 {
                self.0.extend_from_slice(b"0");
            }
        }

        // format milliseconds
        self.0
            .extend_from_slice(itoa::Buffer::new().format(millis).as_bytes());

        self.0.put_u8(b'\n');
        self
    }

    pub fn field_data<T>(mut self, name: &str, data: T) -> EventBuilder
    where
        T: AsRef<str>,
    {
        for line in memchr_split(b'\n', data.as_ref().as_bytes()) {
            self = self.field(name, line)
        }
        self
    }

    pub fn finish(mut self) -> Event {
        self.0.put_u8(b'\n');
        Event(self.0.freeze())
    }
}

/// Server-sent event
#[derive(Debug, Default, Clone)]
pub struct Event(Bytes);

impl Event {
    pub fn combine(events: impl IntoIterator<Item = Event>) -> Event {
        let mut bytes = BytesMut::new();
        for event in events {
            bytes.extend_from_slice(&event.0);
        }
        Event(bytes.freeze())
    }
}

/// Configure the interval between keep-alive messages, the content
/// of each message, and the associated stream.
#[derive(Debug, Clone)]
#[must_use]
pub struct KeepAlive {
    event: Event,
    max_interval: Duration,
}

impl KeepAlive {
    /// Create a new `KeepAlive`.
    pub fn new(event: Event) -> Self {
        Self {
            event,
            max_interval: Duration::from_secs(15),
        }
    }

    /// Customize the interval between keep-alive messages.
    ///
    /// Default is 15 seconds.
    pub fn interval(mut self, time: Duration) -> Self {
        self.max_interval = time;
        self
    }
}

impl Default for KeepAlive {
    fn default() -> Self {
        Self::new(EventBuilder::new().field("", "keep-alive").finish())
    }
}

pin_project! {
    #[derive(Debug)]
    struct KeepAliveStream {
        keep_alive: KeepAlive,
        #[pin]
        alive_timer: Sleep,
    }
}

impl KeepAliveStream {
    fn new(keep_alive: KeepAlive) -> Self {
        Self {
            alive_timer: tokio::time::sleep(keep_alive.max_interval),
            keep_alive,
        }
    }

    fn reset(self: Pin<&mut Self>) {
        let this = self.project();
        this.alive_timer
            .reset(tokio::time::Instant::now() + this.keep_alive.max_interval);
    }

    fn poll_event(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Bytes> {
        let this = self.as_mut().project();

        ready!(this.alive_timer.poll(cx));

        let event = this.keep_alive.event.clone();

        self.reset();

        Poll::Ready(event.0)
    }
}

fn memchr_split(needle: u8, haystack: &[u8]) -> MemchrSplit<'_> {
    MemchrSplit {
        needle,
        haystack: Some(haystack),
    }
}

struct MemchrSplit<'a> {
    needle: u8,
    haystack: Option<&'a [u8]>,
}

impl<'a> Iterator for MemchrSplit<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<Self::Item> {
        let haystack = self.haystack?;
        if let Some(pos) = memchr::memchr(self.needle, haystack) {
            let (front, back) = haystack.split_at(pos);
            self.haystack = Some(&back[1..]);
            Some(front)
        } else {
            self.haystack.take()
        }
    }
}

