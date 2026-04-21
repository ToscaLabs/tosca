use core::future::Future;

use tosca::response::{ResponseKind, SerialResponse as ToscaSerialResponse};
use tosca::route::Route;

use axum::{
    extract::Json,
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde::Serialize;

use super::{BaseResponse, error::ErrorResponse};

/// A response which transmits a JSON message over the network containing
/// the data produced during a device operation.
#[derive(Serialize)]
pub struct SerialResponse<T: Serialize>(ToscaSerialResponse<T>);

impl<T: Serialize> SerialResponse<T> {
    /// Creates a [`SerialResponse`].
    #[must_use]
    pub const fn new(data: T) -> Self {
        Self(ToscaSerialResponse::new(data))
    }
}

impl<T: Serialize> IntoResponse for SerialResponse<T> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self.0)).into_response()
    }
}

mod private {
    #[doc(hidden)]
    pub trait SerialTypeName<Args> {}
}

impl<T, F, Fut> private::SerialTypeName<()> for F
where
    T: Serialize,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<SerialResponse<T>, ErrorResponse>> + Send,
{
}

macro_rules! impl_serial_type_name {
    (
        [$($ty:ident),*], $($last:ident)?
    ) => {
        impl<F, T, Fut, M, $($ty,)* $($last)?> private::SerialTypeName<(M, $($ty,)* $($last)?)> for F
        where
            T: Serialize,
            F: FnOnce($($ty,)* $($last)?) -> Fut,
            Fut: Future<Output = Result<SerialResponse<T>, ErrorResponse>> + Send,
            {
            }
    };
}

super::all_the_tuples!(impl_serial_type_name);

/// Creates a stateful [`BaseResponse`] from a [`SerialResponse`].
#[inline]
pub fn serial_stateful<H, T, S>(route: Route, handler: H) -> impl FnOnce(S) -> BaseResponse
where
    H: Handler<T, S> + private::SerialTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |state: S| BaseResponse::stateful(route, ResponseKind::Serial, handler, state)
}

/// Creates a stateless [`BaseResponse`] from a [`SerialResponse`].
#[inline]
pub fn serial_stateless<H, T, S>(route: Route, handler: H) -> impl FnOnce(S) -> BaseResponse
where
    H: Handler<T, ()> + private::SerialTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |_state: S| BaseResponse::stateless(route, ResponseKind::Serial, handler)
}
