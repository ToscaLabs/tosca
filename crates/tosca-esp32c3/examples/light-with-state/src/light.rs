use tosca::device::DeviceKindTrait;
use tosca::hazards::Hazard;
use tosca::route::Route;

use esp_radio::wifi::WifiDevice;

use tosca_esp32c3::device::Device;
use tosca_esp32c3::devices::builder::DeviceBuilder;
use tosca_esp32c3::parameters::ParametersPayloads;
use tosca_esp32c3::response::{ErrorResponse, InfoResponse, OkResponse, SerialResponse};
use tosca_esp32c3::state::{State, ValueFromRef};

tosca::mandatory_route!(LightOnRoute, "/on", methods: [post, put]);
tosca::mandatory_route!(LightOffRoute, "/off", methods: [post, put]);

// Default main route.
const MAIN_ROUTE: &str = "/light";

// Allowed hazards.
const ALLOWED_HAZARDS: &[Hazard] = &[Hazard::FireHazard, Hazard::ElectricEnergyConsumption];

/// A `light` device.
///
/// Its methods guide in the definition of a correct light.
///
/// The initial placeholder for constructing a [`CompleteLight`].
pub struct Light<S = ()>(DeviceBuilder<S>)
where
    S: ValueFromRef + Send + Sync + 'static;

impl Light<()> {
    /// Creates a [`Light`] without a [`State`].
    #[must_use]
    #[inline]
    pub fn new<K: DeviceKindTrait>(wifi_interface: &WifiDevice<'_>, kind: &K) -> Self {
        Self(DeviceBuilder::new(
            wifi_interface,
            (),
            kind,
            MAIN_ROUTE,
            ALLOWED_HAZARDS,
            2,
            "A light device.",
        ))
    }
}

impl<S> Light<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    /// Creates a [`Light`] with a [`State`].
    #[inline]
    pub fn with_state<K: DeviceKindTrait>(
        wifi_interface: &WifiDevice<'_>,
        state: S,
        kind: &K,
    ) -> Self {
        Self(DeviceBuilder::new(
            wifi_interface,
            state,
            kind,
            MAIN_ROUTE,
            ALLOWED_HAZARDS,
            2,
            "A light device.",
        ))
    }

    /// Turns on a light using a stateless handler, returning an [`OkResponse`]
    /// on success and an [`ErrorResponse`] on failure.
    #[must_use]
    #[inline]
    pub fn turn_light_on_stateless_ok<F, Fut>(
        self,
        route: LightOnRoute,
        func: F,
    ) -> LightOnRouteState<S>
    where
        F: Fn(ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<OkResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        LightOnRouteState(self.0.stateless_ok_route(route.into_route(), func))
    }

    /// Turns on a light using a stateful handler, returning an [`OkResponse`]
    /// on success and an [`ErrorResponse`] on failure.
    #[must_use]
    #[inline]
    pub fn turn_light_on_stateful_ok<F, Fut>(
        self,
        route: LightOnRoute,
        func: F,
    ) -> LightOnRouteState<S>
    where
        F: Fn(State<S>, ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<OkResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        LightOnRouteState(self.0.stateful_ok_route(route.into_route(), func))
    }

    /// Turns on a light using a stateless handler, returning a
    /// [`SerialResponse`] on success and an [`ErrorResponse`] on failure.
    #[must_use]
    #[inline]
    pub fn turn_light_on_stateless_serial<F, Fut>(
        self,
        route: LightOnRoute,
        func: F,
    ) -> LightOnRouteState<S>
    where
        F: Fn(ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<SerialResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        LightOnRouteState(self.0.stateless_serial_route(route.into_route(), func))
    }

    /// Turns on a light using a stateful handler, returning a
    /// [`SerialResponse`] on success and an [`ErrorResponse`] on failure.
    #[must_use]
    #[inline]
    pub fn turn_light_on_stateful_serial<F, Fut>(
        self,
        route: LightOnRoute,
        func: F,
    ) -> LightOnRouteState<S>
    where
        F: Fn(State<S>, ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<SerialResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        LightOnRouteState(self.0.stateful_serial_route(route.into_route(), func))
    }
}

/// A `light` placeholder that includes only the route for turning the light on.
///
/// All methods return a [`CompleteLight`].
pub struct LightOnRouteState<S = ()>(DeviceBuilder<S>)
where
    S: ValueFromRef + Send + Sync + 'static;

impl<S> LightOnRouteState<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    /// Turns off a light using a stateless handler, returning an [`OkResponse`]
    /// on success and an [`ErrorResponse`] on failure.
    #[must_use]
    #[inline]
    pub fn turn_light_off_stateless_ok<F, Fut>(
        self,
        route: LightOffRoute,
        func: F,
    ) -> CompleteLight<S>
    where
        F: Fn(ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<OkResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        CompleteLight(self.0.stateless_ok_route(route.into_route(), func))
    }

    /// Turns off a light using a stateful handler, returning an [`OkResponse`]
    /// on success and an [`ErrorResponse`] on failure.
    #[must_use]
    #[inline]
    pub fn turn_light_off_stateful_ok<F, Fut>(
        self,
        route: LightOffRoute,
        func: F,
    ) -> CompleteLight<S>
    where
        F: Fn(State<S>, ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<OkResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        CompleteLight(self.0.stateful_ok_route(route.into_route(), func))
    }

    /// Turns off a light using a stateless handler, returning a
    /// [`SerialResponse`] on success and an [`ErrorResponse`] on failure.
    #[must_use]
    #[inline]
    pub fn turn_light_off_stateless_serial<F, Fut>(
        self,
        route: LightOffRoute,
        func: F,
    ) -> CompleteLight<S>
    where
        F: Fn(ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<SerialResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        CompleteLight(self.0.stateless_serial_route(route.into_route(), func))
    }

    /// Turns off a light using a stateful handler, returning a
    /// [`SerialResponse`] on success and an [`ErrorResponse`] on failure.
    #[must_use]
    #[inline]
    pub fn turn_light_off_stateful_serial<F, Fut>(
        self,
        route: LightOffRoute,
        func: F,
    ) -> CompleteLight<S>
    where
        F: Fn(State<S>, ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<SerialResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        CompleteLight(self.0.stateful_serial_route(route.into_route(), func))
    }
}

/// A `light` device with methods to turn the light on and off.
pub struct CompleteLight<S = ()>(DeviceBuilder<S>)
where
    S: ValueFromRef + Send + Sync + 'static;

impl<S> CompleteLight<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    /// Sets the main route.
    #[must_use]
    #[inline]
    pub fn main_route(self, main_route: &'static str) -> Self {
        Self(self.0.main_route(main_route))
    }

    /// Adds a [`Route`] with a stateless handler that returns an [`OkResponse`]
    /// on success and an [`ErrorResponse`] on failure.
    #[must_use]
    #[inline]
    pub fn stateless_ok_route<F, Fut>(self, route: Route, func: F) -> Self
    where
        F: Fn(ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<OkResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        Self(self.0.stateless_ok_route(route, func))
    }

    /// Adds a [`Route`] with a stateful handler that returns an [`OkResponse`]
    /// on success and an [`ErrorResponse`] on failure.
    #[must_use]
    #[inline]
    pub fn stateful_ok_route<F, Fut>(self, route: Route, func: F) -> Self
    where
        F: Fn(State<S>, ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<OkResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        Self(self.0.stateful_ok_route(route, func))
    }

    /// Adds a [`Route`] with a stateless handler that returns a
    /// [`SerialResponse`] on success and an [`ErrorResponse`] on failure.
    #[must_use]
    #[inline]
    pub fn stateless_serial_route<F, Fut>(self, route: Route, func: F) -> Self
    where
        F: Fn(ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<SerialResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        Self(self.0.stateless_serial_route(route, func))
    }

    /// Adds a [`Route`] with a stateful handler that returns a
    /// [`SerialResponse`] on success and an [`ErrorResponse`] on failure.
    #[must_use]
    #[inline]
    pub fn stateful_serial_route<F, Fut>(self, route: Route, func: F) -> Self
    where
        F: Fn(State<S>, ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<SerialResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        Self(self.0.stateful_serial_route(route, func))
    }

    /// Adds a [`Route`] with a stateless handler that returns an
    /// [`InfoResponse`] on success and an [`ErrorResponse`] on failure.
    #[must_use]
    #[inline]
    pub fn stateless_info_route<F, Fut>(self, route: Route, func: F) -> Self
    where
        F: Fn(ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<InfoResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        Self(self.0.stateless_info_route(route, func))
    }

    /// Adds a [`Route`] with a stateful handler that returns an
    /// [`InfoResponse`] on success and an [`ErrorResponse`] on failure.
    #[must_use]
    #[inline]
    pub fn stateful_info_route<F, Fut>(self, route: Route, func: F) -> Self
    where
        F: Fn(State<S>, ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<InfoResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        Self(self.0.stateful_info_route(route, func))
    }

    /// Builds a [`Device`].
    ///
    /// **This method consumes the light.**
    #[must_use]
    #[inline]
    pub fn build(self) -> Device<S> {
        self.0.build()
    }
}
