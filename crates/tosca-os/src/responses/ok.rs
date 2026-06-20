use core::future::Future;

use tosca::response::{OkResponse as ToscaOkResponse, ResponseKind};
use tosca::route::Route;

use axum::{
    extract::Json,
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde::Serialize;

use super::{BaseResponse, error::ErrorResponse};

/// A response which transmits a concise JSON message over the network to notify
/// a controller that an operation completed successfully.
#[derive(Serialize)]
pub struct OkResponse(ToscaOkResponse);

impl OkResponse {
    /// Creates an [`OkResponse`].
    #[must_use]
    #[inline]
    pub fn ok() -> Self {
        Self(ToscaOkResponse::ok())
    }
}

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self.0)).into_response()
    }
}

mod private {
    #[doc(hidden)]
    pub trait OkTypeName<Args> {}
}

impl<F, Fut> private::OkTypeName<()> for F
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<OkResponse, ErrorResponse>> + Send,
{
}

macro_rules! impl_ok_type_name {
    (
        [$($ty:ident),*], $($last:ident)?
    ) => {
        impl<F, Fut, M, $($ty,)* $($last)?> private::OkTypeName<(M, $($ty,)* $($last)?)> for F
        where
            F: FnOnce($($ty,)* $($last)?) -> Fut,
            Fut: Future<Output = Result<OkResponse, ErrorResponse>> + Send,
            {
            }
    };
}
super::all_the_tuples!(impl_ok_type_name);

/// Creates a stateful [`BaseResponse`] from an [`OkResponse`].
#[inline]
pub fn ok_stateful<H, T, S>(route: Route, handler: H) -> impl FnOnce(S) -> BaseResponse
where
    H: Handler<T, S> + private::OkTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |state: S| BaseResponse::stateful(route, ResponseKind::Ok, handler, state)
}

/// Creates a stateless [`BaseResponse`] from an [`OkResponse`].
#[inline]
pub fn ok_stateless<H, T, S>(route: Route, handler: H) -> impl FnOnce(S) -> BaseResponse
where
    H: Handler<T, ()> + private::OkTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |_state: S| BaseResponse::stateless(route, ResponseKind::Ok, handler)
}
