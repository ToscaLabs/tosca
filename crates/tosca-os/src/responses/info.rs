use core::future::Future;

use tosca::device::DeviceMetrics;
use tosca::response::{InfoResponse as ToscaInfoResponse, ResponseKind};
use tosca::route::Route;

use axum::{
    extract::Json,
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde::Serialize;

use super::{BaseResponse, error::ErrorResponse};

/// A response which transmits runtime device information as a JSON message
/// over the network.
#[derive(Serialize)]
pub struct InfoResponse(ToscaInfoResponse);

impl InfoResponse {
    /// Creates an [`InfoResponse`].
    #[must_use]
    pub const fn new(metrics: DeviceMetrics) -> Self {
        Self(ToscaInfoResponse::new(metrics))
    }
}

impl IntoResponse for InfoResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self.0)).into_response()
    }
}

mod private {
    #[doc(hidden)]
    pub trait InfoTypeName<Args> {}
}

impl<F, Fut> private::InfoTypeName<()> for F
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<InfoResponse, ErrorResponse>> + Send,
{
}

macro_rules! impl_info_type_name {
    (
        [$($ty:ident),*], $($last:ident)?
    ) => {
        impl<F, Fut, M, $($ty,)* $($last)?> private::InfoTypeName<(M, $($ty,)* $($last)?)> for F
        where
            F: FnOnce($($ty,)* $($last)?) -> Fut,
            Fut: Future<Output = Result<InfoResponse, ErrorResponse>> + Send,
            {
            }
    };
}
super::all_the_tuples!(impl_info_type_name);

/// Creates a stateful [`BaseResponse`] from an [`InfoResponse`].
pub fn info_stateful<H, T, S, I>(route: Route, handler: H) -> impl FnOnce(S, I) -> BaseResponse
where
    H: Handler<T, S> + private::InfoTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
    I: 'static,
{
    move |state: S, _: I| BaseResponse::stateful(route, ResponseKind::Info, handler, state)
}

/// Creates a stateless [`BaseResponse`] from an [`InfoResponse`].
pub fn info_stateless<H, T, S, I>(route: Route, handler: H) -> impl FnOnce(S, I) -> BaseResponse
where
    H: Handler<T, ()> + private::InfoTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
    I: 'static,
{
    move |_state: S, _: I| BaseResponse::stateless(route, ResponseKind::Info, handler)
}
