use tosca::device::{DeviceData, DeviceEnvironment, DeviceKind, DeviceKindId, DeviceKindTrait};
use tosca::route::{RouteConfig, RouteConfigs};

use axum::Router;

use tracing::{info, warn};

use crate::mac::get_mac_addresses;
use crate::responses::BaseResponse;

// Default main route.
const MAIN_ROUTE: &str = "/device";

/// A generic `tosca` device.
///
/// A [`Device`] can only be passed to a [`crate::server::Server`].
#[derive(Debug)]
pub struct Device<S = ()>
where
    S: Clone + Send + Sync + 'static,
{
    // Device main route.
    main_route: &'static str,
    // Device router.
    router: Router,
    // Device state.
    pub(crate) state: S,
    // Device kind.
    kind: DeviceKindId,
    // All device routes along with their associated hazards.
    route_configs: RouteConfigs,
    // Number of mandatory routes.
    num_mandatory_routes: u8,
}

impl Default for Device<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl Device<()> {
    /// Creates a [`Device`] without a state.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self::with_state(())
    }
}

impl<S> Device<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Creates a [`Device`] with the given state.
    #[must_use]
    #[inline]
    pub fn with_state(state: S) -> Self {
        Self::init(&DeviceKind::Unknown, state)
    }

    /// Sets the main route.
    #[must_use]
    pub const fn main_route(mut self, main_route: &'static str) -> Self {
        self.main_route = main_route;
        self
    }

    /// Adds a route to [`Device`].
    #[must_use]
    #[inline]
    pub fn route(self, route: impl FnOnce(S) -> BaseResponse) -> Self {
        let base_response = route(self.state.clone());
        self.response_data(base_response.finalize())
    }

    /// Adds an informative route to [`Device`].
    #[must_use]
    pub fn info_route(self, device_info_route: impl FnOnce(S, ()) -> BaseResponse) -> Self {
        let base_response = device_info_route(self.state.clone(), ());
        self.response_data(base_response.finalize())
    }

    pub(crate) fn init<K: DeviceKindTrait>(kind: &K, state: S) -> Self {
        Self {
            main_route: MAIN_ROUTE,
            router: Router::new(),
            kind: DeviceKindId::from(kind),
            route_configs: RouteConfigs::new(),
            state,
            num_mandatory_routes: 0,
        }
    }

    pub(crate) fn response_data(mut self, data: (RouteConfig, Router)) -> Self {
        self.router = self.router.merge(data.1);
        self.route_configs.add(data.0);
        self
    }

    pub(crate) fn mandatory_response_data<I>(mut self, responses: I) -> Self
    where
        I: IntoIterator<Item = (RouteConfig, Router)>,
    {
        let mut mandatory_routes = RouteConfigs::new();
        for response in responses {
            self.router = self.router.merge(response.1);
            self.num_mandatory_routes += 1;
            mandatory_routes.add(response.0);
        }

        self.route_configs = mandatory_routes.merge(self.route_configs);
        self
    }

    pub(crate) fn finalize(self) -> (&'static str, DeviceData, Router) {
        let (wifi_mac, ethernet_mac) = get_mac_addresses();
        if wifi_mac.is_none() && ethernet_mac.is_none() {
            warn!("Unable to retrieve any Wi-Fi or Ethernet MAC address.");
        }

        for route in &self.route_configs {
            info!(
                "Device route: [{}, \"{}{}\"]",
                route.rest_kind, self.main_route, route.data.path,
            );
        }

        (
            self.main_route,
            DeviceData::new(
                self.kind,
                DeviceEnvironment::Os,
                wifi_mac,
                ethernet_mac,
                self.main_route,
                self.route_configs,
                self.num_mandatory_routes,
            ),
            self.router,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use core::ops::{Deref, DerefMut};

    use tosca::device::DeviceInfo;
    use tosca::energy::Energy;
    use tosca::route::Route;

    use axum::extract::{FromRef, Json, State};

    use serde::{Deserialize, Serialize};

    use tokio::sync::Mutex;

    use crate::responses::error::ErrorResponse;
    use crate::responses::info::{InfoResponse, info_stateful};
    use crate::responses::serial::{SerialResponse, serial_stateful, serial_stateless};

    use super::Device;

    #[derive(Clone)]
    struct DeviceState<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        state: S,
        info: DeviceInfoState,
    }

    impl DeviceState<()> {
        fn empty() -> Self {
            Self::new(())
        }
    }

    impl<S> DeviceState<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        fn new(state: S) -> Self {
            Self {
                state,
                info: DeviceInfoState::new(DeviceInfo::empty()),
            }
        }

        fn add_device_info(mut self, info: DeviceInfo) -> Self {
            self.info = DeviceInfoState::new(info);
            self
        }
    }

    #[derive(Clone)]
    struct SubState {}

    impl FromRef<DeviceState<SubState>> for SubState {
        fn from_ref(device_state: &DeviceState<SubState>) -> SubState {
            device_state.state.clone()
        }
    }

    #[derive(Clone)]
    struct DeviceInfoState {
        info: Arc<Mutex<DeviceInfo>>,
    }

    impl DeviceInfoState {
        fn new(info: DeviceInfo) -> Self {
            Self {
                info: Arc::new(Mutex::new(info)),
            }
        }
    }

    impl Deref for DeviceInfoState {
        type Target = Arc<Mutex<DeviceInfo>>;

        fn deref(&self) -> &Self::Target {
            &self.info
        }
    }

    impl DerefMut for DeviceInfoState {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.info
        }
    }

    impl<S> FromRef<DeviceState<S>> for DeviceInfoState
    where
        S: Clone + Send + Sync + 'static,
    {
        fn from_ref(device_state: &DeviceState<S>) -> DeviceInfoState {
            device_state.info.clone()
        }
    }

    #[derive(Deserialize)]
    struct Inputs {
        parameter: f64,
    }

    #[derive(Serialize, Deserialize)]
    struct DeviceResponse {
        parameter: f64,
    }

    async fn serial_response_with_state(
        State(_state): State<DeviceState<()>>,
        Json(inputs): Json<Inputs>,
    ) -> Result<SerialResponse<DeviceResponse>, ErrorResponse> {
        Ok(SerialResponse::new(DeviceResponse {
            parameter: inputs.parameter,
        }))
    }

    async fn serial_response_with_substate1(
        State(_state): State<SubState>,
        Json(inputs): Json<Inputs>,
    ) -> Result<SerialResponse<DeviceResponse>, ErrorResponse> {
        Ok(SerialResponse::new(DeviceResponse {
            parameter: inputs.parameter,
        }))
    }

    #[derive(Serialize)]
    struct DeviceInfoResponse {
        parameter: f64,
        device_info: DeviceInfo,
    }

    // This method demonstrates the library flexibility, but its use is
    // not recommended.
    // A controller may ignore a DeviceInfo structure if it is contained
    // within a SerialResponse.
    async fn serial_response_with_substate2(
        State(state): State<DeviceInfoState>,
        Json(inputs): Json<Inputs>,
    ) -> Result<SerialResponse<DeviceInfoResponse>, ErrorResponse> {
        // Retrieve the internal state.
        let mut device_info = state.lock().await;

        // Change the state.
        device_info.energy = Energy::empty();

        Ok(SerialResponse::new(DeviceInfoResponse {
            parameter: inputs.parameter,
            device_info: device_info.clone(),
        }))
    }

    async fn info_response_with_substate3(
        State(state): State<DeviceInfoState>,
    ) -> Result<InfoResponse, ErrorResponse> {
        // Retrieve the internal state.
        let mut device_info = state.lock().await;

        // Change the state.
        device_info.energy = Energy::empty();

        Ok(InfoResponse::new(device_info.clone()))
    }

    async fn serial_response_without_state(
        Json(inputs): Json<Inputs>,
    ) -> Result<SerialResponse<DeviceResponse>, ErrorResponse> {
        Ok(SerialResponse::new(DeviceResponse {
            parameter: inputs.parameter,
        }))
    }

    struct AllRoutes {
        with_state_route: Route,
        without_state_route: Route,
    }

    #[inline]
    fn create_routes() -> AllRoutes {
        AllRoutes {
            with_state_route: Route::put("State response", "/state-response")
                .description("Run response with state."),

            without_state_route: Route::post("No state route", "/no-state-route")
                .description("Run response without state."),
        }
    }

    #[test]
    fn with_state() {
        let routes = create_routes();

        let state = DeviceState::empty().add_device_info(DeviceInfo::empty());

        let _ = Device::with_state(state)
            .route(serial_stateful(
                routes.with_state_route,
                serial_response_with_state,
            ))
            .route(serial_stateless(
                routes.without_state_route,
                serial_response_without_state,
            ));
    }

    #[test]
    fn with_substates() {
        let routes = create_routes();

        let state = DeviceState::new(SubState {}).add_device_info(DeviceInfo::empty());

        let _ = Device::with_state(state)
            .route(serial_stateful(
                routes.with_state_route,
                serial_response_with_substate1,
            ))
            .route(serial_stateful(
                Route::put("Substate response", "/substate-response")
                    .description("Run a serial response with a substate."),
                serial_response_with_substate2,
            ))
            .info_route(info_stateful(
                Route::put("Substate info", "/substate-info")
                    .description("Run an informative response with a substate."),
                info_response_with_substate3,
            ))
            .route(serial_stateless(
                routes.without_state_route,
                serial_response_without_state,
            ));
    }

    #[test]
    fn without_state() {
        let routes = create_routes();

        let _ = Device::new().route(serial_stateless(
            routes.without_state_route,
            serial_response_without_state,
        ));
    }
}
