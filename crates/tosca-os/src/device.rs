use std::borrow::Cow;

use tosca::device::{DeviceDescription, DeviceEnvironment, DeviceScheme};
use tosca::route::{RouteConfig, RouteConfigs};

use axum::Router;

use tracing::{error, info};

use crate::error::{Error, ErrorKind, Result};
use crate::mac::get_mac_addresses;
use crate::responses::BaseResponse;

// Default main route.
const MAIN_ROUTE: &str = "/device";

/// A `tosca` device.
///
/// A [`Device`] can only be passed to a [`crate::server::Server`].
#[derive(Debug)]
pub struct Device<S = ()>
where
    S: Clone + Send + Sync + 'static,
{
    // Device description.
    description: DeviceDescription,
    // Device main route.
    main_route: &'static str,
    // Device router.
    router: Router,
    // Device state.
    state: S,
}

impl Device<()> {
    /// Creates a [`Device`] without a state.
    #[must_use]
    #[inline]
    pub fn new(scheme: DeviceScheme) -> Self {
        Self::with_state(scheme, ())
    }
}

impl<S> Device<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Creates a [`Device`] with the given state.
    #[must_use]
    #[inline]
    pub fn with_state(scheme: DeviceScheme, state: S) -> Self {
        Self::init(scheme, state)
    }

    /// Sets the main route.
    #[must_use]
    pub const fn main_route(mut self, main_route: &'static str) -> Self {
        self.main_route = main_route;
        self
    }

    /// Sets the device description.
    #[must_use]
    #[inline]
    pub fn description(mut self, description: &'static str) -> Self {
        self.description.data.description = Some(Cow::Borrowed(description));
        self
    }

    /// Adds a route to [`Device`].
    #[must_use]
    #[inline]
    pub fn route(self, route: impl FnOnce(S) -> BaseResponse) -> Self {
        let base_response = route(self.state.clone());
        let response = base_response.finalize(self.description.data.scheme.allowed_hazards());
        self.response_data(response)
    }

    /// Adds an informative route to [`Device`].
    #[must_use]
    pub fn info_route(self, device_info_route: impl FnOnce(S, ()) -> BaseResponse) -> Self {
        let base_response = device_info_route(self.state.clone(), ());
        let response = base_response.finalize(self.description.data.scheme.allowed_hazards());
        self.response_data(response)
    }

    /// Builds a [`DeviceVerified`].
    ///
    /// # Errors
    ///
    /// - Returns an error if no **Wi-Fi** or **Ethernet** MAC address is
    ///   available for the device ID.
    /// - Returns an error if mandatory routes are missing or invalid.
    pub fn build(mut self) -> Result<DeviceVerified> {
        let (wifi_mac, ethernet_mac) = get_mac_addresses();
        if wifi_mac.is_none() && ethernet_mac.is_none() {
            let message = "No Wi-Fi or Ethernet MAC address is available for the device ID";
            error!(message);
            return Err(Error::new(ErrorKind::NoIdFound, message));
        }

        if let Some(missing_route) =
            self.description
                .data
                .scheme
                .mandatory_routes()
                .iter()
                .find(|&mandatory_route| {
                    !self
                        .description
                        .route_configs
                        .iter()
                        .any(|route| route.data.path == *mandatory_route)
                })
        {
            let message = format!("The mandatory route `{missing_route}` is missing");
            error!(message);
            return Err(Error::new(ErrorKind::MandatoryRoutes, message));
        }

        self.description.data.wifi_mac = wifi_mac;
        self.description.data.ethernet_mac = ethernet_mac;
        self.description.main_route = self.main_route.into();

        Ok(DeviceVerified {
            description: self.description,
            main_route: self.main_route,
            router: self.router,
        })
    }

    pub(crate) fn init(scheme: DeviceScheme, state: S) -> Self {
        let description = DeviceDescription::new(scheme, MAIN_ROUTE, RouteConfigs::new())
            .environment(DeviceEnvironment::Os);
        Self {
            description,
            main_route: MAIN_ROUTE,
            router: Router::new(),
            state,
        }
    }

    pub(crate) fn response_data(mut self, data: (RouteConfig, Router)) -> Self {
        self.router = self.router.merge(data.1);
        self.description.route_configs.add(data.0);
        self
    }
}

/// A device with fully validated data.
///
/// [`DeviceVerified`] is a transient type, intended solely for passing
/// validated device data to [`crate::server::Server`].
#[derive(Debug)]
pub struct DeviceVerified {
    // Device description.
    description: DeviceDescription,
    // Device main route.
    main_route: &'static str,
    // Device router.
    router: Router,
}

impl DeviceVerified {
    #[inline]
    pub(crate) fn finalize(self) -> (&'static str, DeviceDescription, Router) {
        for route in &self.description.route_configs {
            info!(
                "Device route: [{}, \"{}{}\"]",
                route.rest_kind, self.main_route, route.data.path,
            );
        }

        (self.main_route, self.description, self.router)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use core::ops::{Deref, DerefMut};

    use tosca::device::schemes::LIGHT_SCHEME;
    use tosca::device::{DeviceMetrics, DeviceScheme};
    use tosca::energy::Energy;
    use tosca::route::Route;

    use axum::extract::{FromRef, Json, State};

    use serde::{Deserialize, Serialize};

    use tracing::error;

    use crate::error::{Error, ErrorKind, Result};

    use crate::responses::error::ErrorResponse;
    use crate::responses::info::{InfoResponse, info_stateful};
    use crate::responses::serial::{SerialResponse, serial_stateful, serial_stateless};

    use super::{Device, DeviceVerified};

    #[derive(Clone)]
    struct DeviceState<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        state: S,
        info_state: DeviceInfoState,
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
                info_state: DeviceInfoState::new(DeviceMetrics::with_energy(Energy::empty())),
            }
        }

        fn add_device_info(self, metrics: DeviceMetrics) -> Self {
            match self.info_state.lock() {
                Ok(mut info_state) => *info_state = metrics,
                Err(e) => error!("Failed to obtain info state, leaving state unchanged: {e}"),
            }
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
        info: Arc<Mutex<DeviceMetrics>>,
    }

    impl DeviceInfoState {
        fn new(metrics: DeviceMetrics) -> Self {
            Self {
                info: Arc::new(Mutex::new(metrics)),
            }
        }
    }

    impl Deref for DeviceInfoState {
        type Target = Arc<Mutex<DeviceMetrics>>;

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
            device_state.info_state.clone()
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
    ) -> std::result::Result<SerialResponse<DeviceResponse>, ErrorResponse> {
        Ok(SerialResponse::new(DeviceResponse {
            parameter: inputs.parameter,
        }))
    }

    async fn serial_response_with_substate1(
        State(_state): State<SubState>,
        Json(inputs): Json<Inputs>,
    ) -> std::result::Result<SerialResponse<DeviceResponse>, ErrorResponse> {
        Ok(SerialResponse::new(DeviceResponse {
            parameter: inputs.parameter,
        }))
    }

    async fn info_response_with_substate2(
        State(state): State<DeviceInfoState>,
    ) -> std::result::Result<InfoResponse, ErrorResponse> {
        // Retrieve the internal state.
        let mut device_info = state.lock().map_err(|e| {
            ErrorResponse::internal_with_error("Failed to obtain state lock", &e.to_string())
        })?;

        // Change the state.
        device_info.energy = Energy::empty();

        Ok(InfoResponse::new(device_info.clone()))
    }

    async fn serial_response_without_state(
        Json(inputs): Json<Inputs>,
    ) -> std::result::Result<SerialResponse<DeviceResponse>, ErrorResponse> {
        Ok(SerialResponse::new(DeviceResponse {
            parameter: inputs.parameter,
        }))
    }

    struct AllRoutes {
        route1: Route,
        route2: Route,
    }

    #[inline]
    fn light_routes() -> AllRoutes {
        AllRoutes {
            route1: Route::put("Light on", "/on").description("Turn light on."),

            route2: Route::post("Light off", "/off").description("Turn light off"),
        }
    }

    #[inline]
    fn custom_routes() -> AllRoutes {
        AllRoutes {
            route1: Route::put("Switch on", "/switch-on").description("Switch something on."),

            route2: Route::post("Switch off", "/switch-off").description("Switch something off"),
        }
    }

    #[inline]
    fn state_routes() -> AllRoutes {
        AllRoutes {
            route1: Route::put("State response", "/state-response")
                .description("Run response with state."),

            route2: Route::post("No state route", "/no-state-route")
                .description("Run response without state."),
        }
    }

    #[inline]
    fn create_device_for_routes(routes: AllRoutes, scheme: DeviceScheme) -> Result<DeviceVerified> {
        Device::new(scheme)
            .route(serial_stateless(
                routes.route1,
                serial_response_without_state,
            ))
            .route(serial_stateless(
                routes.route2,
                serial_response_without_state,
            ))
            .build()
    }

    #[test]
    fn test_light_device() {
        assert!(create_device_for_routes(light_routes(), LIGHT_SCHEME).is_ok());
    }

    #[test]
    fn test_light_device_with_wrong_routes() {
        let device_verified = create_device_for_routes(custom_routes(), LIGHT_SCHEME);
        assert!(device_verified.map_or_else(
            |e| e
                == Error::new(
                    ErrorKind::MandatoryRoutes,
                    "The mandatory route `/on` is missing"
                ),
            |_| false
        ));
    }

    #[test]
    fn test_custom_device() {
        assert!(
            create_device_for_routes(custom_routes(), DeviceScheme::base_custom_scheme("Light"))
                .is_ok()
        );
    }

    #[test]
    fn test_with_state() {
        let routes = state_routes();
        let scheme = DeviceScheme::base_custom_scheme("Light");
        let state = DeviceState::empty();

        let device_verified = Device::with_state(scheme, state)
            .route(serial_stateful(routes.route1, serial_response_with_state))
            .route(serial_stateless(
                routes.route2,
                serial_response_without_state,
            ))
            .build();

        assert!(device_verified.is_ok());
    }

    #[test]
    fn test_with_substates() {
        let routes = state_routes();
        let scheme = DeviceScheme::base_custom_scheme("Light");
        let state = DeviceState::new(SubState {})
            .add_device_info(DeviceMetrics::with_energy(Energy::empty()));

        let device_verified = Device::with_state(scheme, state)
            .route(serial_stateful(
                routes.route1,
                serial_response_with_substate1,
            ))
            .info_route(info_stateful(
                Route::put("Substate info", "/substate-info")
                    .description("Run an informative response with a substate."),
                info_response_with_substate2,
            ))
            .route(serial_stateless(
                routes.route2,
                serial_response_without_state,
            ))
            .build();

        assert!(device_verified.is_ok());
    }

    #[test]
    fn test_without_state() {
        assert!(
            create_device_for_routes(state_routes(), DeviceScheme::base_custom_scheme("Light"))
                .is_ok()
        );
    }
}
