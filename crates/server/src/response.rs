use std::{
    convert::Infallible,
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::ready;
use pin_project::pin_project;
use serde::Serialize;
use sithra_transport::datapack::DataPack;
use tower::Service;
use ulid::Ulid;

use crate::{extract::payload::Payload, request::Request};

pub struct Response {
    pub data: Option<DataPack>,
}

pub struct Error<E: ToString>(E);

impl<S> From<S> for Error<S>
where
    S: ToString,
{
    fn from(value: S) -> Self {
        Self(value)
    }
}

impl Response {
    #[must_use]
    pub const fn new(data: DataPack) -> Self {
        Self { data: Some(data) }
    }

    #[must_use]
    pub const fn none() -> Self {
        Self { data: None }
    }

    #[must_use]
    pub const fn is_none(&self) -> bool {
        self.data.is_none()
    }

    pub const fn correlate(&mut self, id: Ulid) {
        if let Some(data) = self.data.as_mut() {
            data.correlate(id);
        }
    }

    pub fn error(error: &impl ToString) -> Self {
        Self {
            data: Some(DataPack::builder().build_with_error(error)),
        }
    }
}

pub trait IntoResponse {
    /// Create a response.
    #[must_use]
    fn into_response(self) -> Response;
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        Response { data: None }
    }
}

impl IntoResponse for Infallible {
    fn into_response(self) -> Response {
        match self {}
    }
}

impl IntoResponse for DataPack {
    fn into_response(self) -> Response {
        Response { data: Some(self) }
    }
}

impl<S> IntoResponse for Error<S>
where
    S: ToString,
{
    fn into_response(self) -> Response {
        Response::error(&self.0)
    }
}

impl<V, E> IntoResponse for Result<V, E>
where
    V: IntoResponse,
    E: ToString,
{
    fn into_response(self) -> Response {
        match self {
            Ok(value) => value.into_response(),
            Err(error) => Response::error(&error),
        }
    }
}

impl<V: Serialize> IntoResponse for Payload<V> {
    fn into_response(self) -> Response {
        let Self(payload) = self;
        let value = rmpv::ext::to_value(payload);
        let Ok(value) = value else {
            return Response::error(&"Failed to serialize payload");
        };
        DataPack::builder().build_with_payload(value).into_response()
    }
}

#[derive(Clone)]
pub(crate) struct MapIntoResponse<S> {
    inner: S,
}

impl<S> MapIntoResponse<S> {
    pub(crate) const fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S> Service<Request> for MapIntoResponse<S>
where
    S: Service<Request>,
    S::Response: IntoResponse,
{
    type Error = S::Error;
    type Future = MapIntoResponseFuture<S::Future>;
    type Response = Response;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        MapIntoResponseFuture {
            inner: self.inner.call(req),
        }
    }
}

#[pin_project]
pub(crate) struct MapIntoResponseFuture<F> {
    #[pin]
    inner: F,
}

impl<F, T, E> Future for MapIntoResponseFuture<F>
where
    F: Future<Output = Result<T, E>>,
    T: IntoResponse,
{
    type Output = Result<Response, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let res = ready!(self.project().inner.poll(cx)?);
        Poll::Ready(Ok(res.into_response()))
    }
}
