/// A response providing details about an error encountered during a
/// device operation.
pub mod error;
/// A response containing a device energy and economy information.
pub mod info;
/// A response notifying the controller that an operation completed
/// successfully.
pub mod ok;
/// A response containing the data produced during a device operation.
pub mod serial;
/// A response containing a byte stream of data.
#[cfg(feature = "stream")]
pub mod stream;

use tosca::hazards::Hazard;
use tosca::parameters::Parameters;
use tosca::response::ResponseKind;
use tosca::route::{RestKind, Route, RouteConfig};

use axum::{Router, handler::Handler};

use tracing::info;

#[rustfmt::skip]
macro_rules! all_the_tuples {
    ($name:ident) => {
        $name!([], );
        $name!([], T1);
        $name!([T1], T2);
        $name!([T1, T2], T3);
        $name!([T1, T2, T3], T4);
        $name!([T1, T2, T3, T4], T5);
        $name!([T1, T2, T3, T4, T5], T6);
        $name!([T1, T2, T3, T4, T5, T6], T7);
        $name!([T1, T2, T3, T4, T5, T6, T7], T8);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8], T9);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9], T10);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10], T11);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11], T12);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12], T13);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13], T14);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14], T15);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15], T16);
    };
}

pub(super) use all_the_tuples;

fn build_get_route(route: &str, parameters: &Parameters) -> String {
    let mut route = String::from(route);
    for name in parameters.names() {
        let append_str = format!("/{{{name}}}");
        route.push_str(&append_str);
    }
    info!("Build GET route: {}", route);
    route
}

#[derive(Debug)]
/// A base response for a [`crate::device::Device`].
///
/// All responses are converted into this base response.
pub struct BaseResponse {
    // Router.
    pub(crate) router: Router,
    // Route configuration.
    pub(crate) route: Route,
    // Response kind.
    response_kind: ResponseKind,
}

impl BaseResponse {
    pub(crate) fn stateless<H, T>(route: Route, response_kind: ResponseKind, handler: H) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        Self::init(route, response_kind, handler, ())
    }

    pub(crate) fn stateful<H, T, S>(
        route: Route,
        response_kind: ResponseKind,
        handler: H,
        state: S,
    ) -> Self
    where
        H: Handler<T, S>,
        T: 'static,
        S: Clone + Send + Sync + 'static,
    {
        Self::init(route, response_kind, handler, state)
    }

    fn init<H, T, S>(route: Route, response_kind: ResponseKind, handler: H, state: S) -> Self
    where
        H: Handler<T, S>,
        T: 'static,
        S: Clone + Send + Sync + 'static,
    {
        // Create the GET route for the axum architecture.
        let route_str = if matches!(route.kind(), RestKind::Get) && !route.parameters().is_empty() {
            &build_get_route(route.route(), route.parameters())
        } else {
            route.route()
        };

        let router = Router::new()
            .route(
                route_str,
                match route.kind() {
                    RestKind::Get => axum::routing::get(handler),
                    RestKind::Put => axum::routing::put(handler),
                    RestKind::Post => axum::routing::post(handler),
                    RestKind::Delete => axum::routing::delete(handler),
                },
            )
            .with_state(state);

        Self {
            router,
            route,
            response_kind,
        }
    }

    pub(crate) fn finalize(self, allowed_hazards: &[Hazard]) -> (RouteConfig, Router) {
        let route = if allowed_hazards.is_empty() {
            self.route.serialize_data()
        } else {
            self.route
                .remove_prohibited_hazards(allowed_hazards)
                .serialize_data()
        };

        (route.change_response_kind(self.response_kind), self.router)
    }
}

#[cfg(test)]
mod tests {
    use tosca::parameters::Parameters;

    use super::{Route, build_get_route};

    #[test]
    fn test_build_get_route() {
        let route = Route::get("Route", "/route")
            .description("A GET route.")
            .with_parameters(
                Parameters::new()
                    .rangeu64_with_default("rangeu64", (0, 20, 1), 5)
                    .rangef64("rangef64", (0., 20., 0.1)),
            );

        assert_eq!(
            &build_get_route(route.route(), route.parameters()),
            "/route/{rangeu64}/{rangef64}"
        );
    }
}
