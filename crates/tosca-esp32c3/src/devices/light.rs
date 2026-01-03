use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::vec::Vec;

use tosca::device::{DeviceData, DeviceEnvironment, DeviceKind};
use tosca::hazards::Hazard;
use tosca::response::ResponseKind;
use tosca::route::{Route, RouteConfigs};

use esp_radio::wifi::WifiDevice;

use log::error;

use crate::device::Device;
use crate::parameters::ParametersPayloads;
use crate::response::{ErrorResponse, InfoResponse, OkResponse, SerialResponse};
use crate::server::{
    FuncIndex, FuncType, Functions, InfoFn, InfoStateFn, OkFn, OkStateFn, SerialFn, SerialStateFn,
};
use crate::state::{State, ValueFromRef};

// Default main route.
const MAIN_ROUTE: &str = "/light";

// Allowed hazards.
const ALLOWED_HAZARDS: &[Hazard] = &[Hazard::FireHazard, Hazard::ElectricEnergyConsumption];

/// A `light` device.
///
/// Its methods guide in the definition of a correct light.
///
/// The initial placeholder for constructing a [`CompleteLight`].
pub struct Light<S = ()>(CompleteLight<S>)
where
    S: ValueFromRef + Send + Sync + 'static;

impl Light<()> {
    /// Creates a [`Light`] without a [`State`].
    #[must_use]
    #[inline]
    pub fn new(wifi_interface: &WifiDevice<'_>) -> Self {
        Self(CompleteLight::with_state(wifi_interface, ()))
    }
}

impl<S> Light<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    /// Creates a [`Light`] with a [`State`].
    #[inline]
    pub fn with_state(wifi_interface: &WifiDevice<'_>, state: S) -> Self {
        Self(CompleteLight::with_state(wifi_interface, state))
    }

    /// Turns on a light using a stateless handler, returning an [`OkResponse`]
    /// on success and an [`ErrorResponse`] on failure.
    #[must_use]
    #[inline]
    pub fn turn_light_on_stateless_ok<F, Fut>(
        self,
        route: tosca::route::LightOnRoute,
        func: F,
    ) -> LightOnRoute<S>
    where
        F: Fn(ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<OkResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        LightOnRoute(self.0.stateless_ok_route(route.into_route(), func))
    }

    /// Turns on a light using a stateful handler, returning an [`OkResponse`]
    /// on success and an [`ErrorResponse`] on failure.
    #[must_use]
    #[inline]
    pub fn turn_light_on_stateful_ok<F, Fut>(
        self,
        route: tosca::route::LightOnRoute,
        func: F,
    ) -> LightOnRoute<S>
    where
        F: Fn(State<S>, ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<OkResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        LightOnRoute(self.0.stateful_ok_route(route.into_route(), func))
    }

    /// Turns on a light using a stateless handler, returning a
    /// [`SerialResponse`] on success and an [`ErrorResponse`] on failure.
    #[must_use]
    #[inline]
    pub fn turn_light_on_stateless_serial<F, Fut>(
        self,
        route: tosca::route::LightOnRoute,
        func: F,
    ) -> LightOnRoute<S>
    where
        F: Fn(ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<SerialResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        LightOnRoute(self.0.stateless_serial_route(route.into_route(), func))
    }

    /// Turns on a light using a stateful handler, returning a
    /// [`SerialResponse`] on success and an [`ErrorResponse`] on failure.
    #[must_use]
    #[inline]
    pub fn turn_light_on_stateful_serial<F, Fut>(
        self,
        route: tosca::route::LightOnRoute,
        func: F,
    ) -> LightOnRoute<S>
    where
        F: Fn(State<S>, ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<SerialResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        LightOnRoute(self.0.stateful_serial_route(route.into_route(), func))
    }
}

/// A `light` placeholder that includes only the route for turning the light on.
///
/// All methods return a [`CompleteLight`].
pub struct LightOnRoute<S = ()>(CompleteLight<S>)
where
    S: ValueFromRef + Send + Sync + 'static;

impl<S> LightOnRoute<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    /// Turns off a light using a stateless handler, returning an [`OkResponse`]
    /// on success and an [`ErrorResponse`] on failure.
    #[must_use]
    #[inline]
    pub fn turn_light_off_stateless_ok<F, Fut>(
        self,
        route: tosca::route::LightOffRoute,
        func: F,
    ) -> CompleteLight<S>
    where
        F: Fn(ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<OkResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        self.0.stateless_ok_route(route.into_route(), func)
    }

    /// Turns off a light using a stateful handler, returning an [`OkResponse`]
    /// on success and an [`ErrorResponse`] on failure.
    #[must_use]
    #[inline]
    pub fn turn_light_off_stateful_ok<F, Fut>(
        self,
        route: tosca::route::LightOffRoute,
        func: F,
    ) -> CompleteLight<S>
    where
        F: Fn(State<S>, ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<OkResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        self.0.stateful_ok_route(route.into_route(), func)
    }

    /// Turns off a light using a stateless handler, returning a
    /// [`SerialResponse`] on success and an [`ErrorResponse`] on failure.
    #[must_use]
    #[inline]
    pub fn turn_light_off_stateless_serial<F, Fut>(
        self,
        route: tosca::route::LightOffRoute,
        func: F,
    ) -> CompleteLight<S>
    where
        F: Fn(ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<SerialResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        self.0.stateless_serial_route(route.into_route(), func)
    }

    /// Turns off a light using a stateful handler, returning a
    /// [`SerialResponse`] on success and an [`ErrorResponse`] on failure.
    #[must_use]
    #[inline]
    pub fn turn_light_off_stateful_serial<F, Fut>(
        self,
        route: tosca::route::LightOffRoute,
        func: F,
    ) -> CompleteLight<S>
    where
        F: Fn(State<S>, ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<SerialResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        self.0.stateful_serial_route(route.into_route(), func)
    }
}

/// A `light` device with methods to turn the light on and off.
pub struct CompleteLight<S = ()>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    wifi_mac: [u8; 6],
    main_route: &'static str,
    state: State<S>,
    routes_functions: Functions<S>,
    device_data: DeviceData,
    index_array: Vec<FuncIndex>,
}

impl<S> CompleteLight<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    /// Sets the main route.
    #[must_use]
    #[inline]
    pub fn main_route(mut self, main_route: &'static str) -> Self {
        self.main_route = main_route;
        self.device_data.main_route = Cow::Borrowed(main_route);
        self
    }

    /// Adds a [`Route`] with a stateless handler that returns an [`OkResponse`]
    /// on success and an [`ErrorResponse`] on failure.
    #[must_use]
    pub fn stateless_ok_route<F, Fut>(self, route: Route, func: F) -> Self
    where
        F: Fn(ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<OkResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        self.route_func_manager(route, ResponseKind::Ok, move |mut func_manager| {
            let func: OkFn = Box::new(move |parameters_values| Box::pin(func(parameters_values)));
            func_manager.routes_functions.0.push(func);
            func_manager.index_array.push(FuncIndex::new(
                FuncType::OkStateless,
                func_manager.routes_functions.0.len() - 1,
            ));
            func_manager
        })
    }

    /// Adds a [`Route`] with a stateful handler that returns an [`OkResponse`]
    /// on success and an [`ErrorResponse`] on failure.
    #[must_use]
    pub fn stateful_ok_route<F, Fut>(self, route: Route, func: F) -> Self
    where
        F: Fn(State<S>, ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<OkResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        self.route_func_manager(route, ResponseKind::Ok, move |mut func_manager| {
            let func: OkStateFn<S> =
                Box::new(move |state, parameters_values| Box::pin(func(state, parameters_values)));
            func_manager.routes_functions.1.push(func);
            func_manager.index_array.push(FuncIndex::new(
                FuncType::OkStateful,
                func_manager.routes_functions.1.len() - 1,
            ));
            func_manager
        })
    }

    /// Adds a [`Route`] with a stateless handler that returns a
    /// [`SerialResponse`] on success and an [`ErrorResponse`] on failure.
    #[must_use]
    pub fn stateless_serial_route<F, Fut>(self, route: Route, func: F) -> Self
    where
        F: Fn(ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<SerialResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        self.route_func_manager(route, ResponseKind::Serial, move |mut func_manager| {
            let func: SerialFn =
                Box::new(move |parameters_values| Box::pin(func(parameters_values)));
            func_manager.routes_functions.2.push(func);
            func_manager.index_array.push(FuncIndex::new(
                FuncType::SerialStateless,
                func_manager.routes_functions.2.len() - 1,
            ));
            func_manager
        })
    }

    /// Adds a [`Route`] with a stateful handler that returns a
    /// [`SerialResponse`] on success and an [`ErrorResponse`] on failure.
    #[must_use]
    pub fn stateful_serial_route<F, Fut>(self, route: Route, func: F) -> Self
    where
        F: Fn(State<S>, ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<SerialResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        self.route_func_manager(route, ResponseKind::Serial, move |mut func_manager| {
            let func: SerialStateFn<S> =
                Box::new(move |state, parameters_values| Box::pin(func(state, parameters_values)));
            func_manager.routes_functions.3.push(func);
            func_manager.index_array.push(FuncIndex::new(
                FuncType::SerialStateful,
                func_manager.routes_functions.3.len() - 1,
            ));
            func_manager
        })
    }

    /// Adds a [`Route`] with a stateless handler that returns an
    /// [`InfoResponse`] on success and an [`ErrorResponse`] on failure.
    #[must_use]
    pub fn stateless_info_route<F, Fut>(self, route: Route, func: F) -> Self
    where
        F: Fn(ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<InfoResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        self.route_func_manager(route, ResponseKind::Info, move |mut func_manager| {
            let func: InfoFn = Box::new(move |parameters_values| Box::pin(func(parameters_values)));
            func_manager.routes_functions.4.push(func);
            func_manager.index_array.push(FuncIndex::new(
                FuncType::InfoStateless,
                func_manager.routes_functions.4.len() - 1,
            ));
            func_manager
        })
    }

    /// Adds a [`Route`] with a stateful handler that returns an
    /// [`InfoResponse`] on success and an [`ErrorResponse`] on failure.
    #[must_use]
    pub fn stateful_info_route<F, Fut>(self, route: Route, func: F) -> Self
    where
        F: Fn(State<S>, ParametersPayloads) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<InfoResponse, ErrorResponse>> + Send + Sync + 'static,
    {
        self.route_func_manager(route, ResponseKind::Info, move |mut func_manager| {
            let func: InfoStateFn<S> =
                Box::new(move |state, parameters_values| Box::pin(func(state, parameters_values)));
            func_manager.routes_functions.5.push(func);
            func_manager.index_array.push(FuncIndex::new(
                FuncType::InfoStateful,
                func_manager.routes_functions.5.len() - 1,
            ));
            func_manager
        })
    }

    /// Builds a [`Device`].
    ///
    /// **This method consumes the light.**
    #[must_use]
    #[inline]
    pub fn build(self) -> Device<S> {
        Device::new(
            self.wifi_mac,
            self.state,
            self.device_data,
            self.main_route,
            self.routes_functions,
            self.index_array,
        )
    }

    fn route_func_manager<F>(
        mut self,
        route: Route,
        response_kind: ResponseKind,
        add_async_function: F,
    ) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        let route_config = route
            .remove_prohibited_hazards(ALLOWED_HAZARDS)
            .serialize_data()
            .change_response_kind(response_kind);

        if self.device_data.route_configs.contains(&route_config) {
            error!(
                "The route with prefix `{}` already exists!",
                route_config.data.path
            );
            return self;
        }

        self.device_data.route_configs.add(route_config);

        add_async_function(self)
    }

    #[inline]
    fn with_state(wifi_interface: &WifiDevice<'_>, state: S) -> Self {
        let wifi_mac = wifi_interface.mac_address();

        let device_data = DeviceData::new(
            DeviceKind::Light,
            DeviceEnvironment::Esp32,
            None,
            None,
            MAIN_ROUTE,
            RouteConfigs::new(),
            2,
        )
        .description("A light device.");

        Self {
            wifi_mac,
            main_route: MAIN_ROUTE,
            state: State(state),
            routes_functions: (
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ),
            device_data,
            index_array: Vec::new(),
        }
    }
}
