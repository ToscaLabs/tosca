use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::vec::Vec;

use tosca::device::{DeviceData, DeviceEnvironment, DeviceKindId, DeviceKindTrait};
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

/// A generic device builder.
///
/// Contains all the shared logic for registering routes and building
/// a [`Device`]. Device-specific modules (light, relay, etc.) wrap this
/// builder and add typestate enforcement for their mandatory routes.
pub struct DeviceBuilder<S = ()>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    wifi_mac: [u8; 6],
    main_route: &'static str,
    allowed_hazards: &'static [Hazard],
    state: State<S>,
    routes_functions: Functions<S>,
    device_data: DeviceData,
    index_array: Vec<FuncIndex>,
}

impl<S> DeviceBuilder<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    /// Creates a new device builder.
    ///
    /// The generic `K` is erased here — it converts to a [`DeviceKindId`]
    /// stored in [`DeviceData`]. No generic propagation beyond this call.
    #[allow(clippy::too_many_arguments)]
    pub fn new<K: DeviceKindTrait>(
        wifi_interface: &WifiDevice<'_>,
        state: S,
        kind: &K,
        main_route: &'static str,
        allowed_hazards: &'static [Hazard],
        mandatory_routes: u8,
        description: &'static str,
    ) -> Self {
        let wifi_mac = wifi_interface.mac_address();

        let device_data = DeviceData::new(
            DeviceKindId::from(kind),
            DeviceEnvironment::Esp32,
            None,
            None,
            main_route,
            RouteConfigs::new(),
            mandatory_routes,
        )
        .description(description);

        Self {
            wifi_mac,
            main_route,
            allowed_hazards,
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
        self.route_func_manager(route, ResponseKind::Ok, move |mut builder| {
            let func: OkFn = Box::new(move |parameters_values| Box::pin(func(parameters_values)));
            builder.routes_functions.0.push(func);
            builder.index_array.push(FuncIndex::new(
                FuncType::OkStateless,
                builder.routes_functions.0.len() - 1,
            ));
            builder
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
        self.route_func_manager(route, ResponseKind::Ok, move |mut builder| {
            let func: OkStateFn<S> =
                Box::new(move |state, parameters_values| Box::pin(func(state, parameters_values)));
            builder.routes_functions.1.push(func);
            builder.index_array.push(FuncIndex::new(
                FuncType::OkStateful,
                builder.routes_functions.1.len() - 1,
            ));
            builder
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
        self.route_func_manager(route, ResponseKind::Serial, move |mut builder| {
            let func: SerialFn =
                Box::new(move |parameters_values| Box::pin(func(parameters_values)));
            builder.routes_functions.2.push(func);
            builder.index_array.push(FuncIndex::new(
                FuncType::SerialStateless,
                builder.routes_functions.2.len() - 1,
            ));
            builder
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
        self.route_func_manager(route, ResponseKind::Serial, move |mut builder| {
            let func: SerialStateFn<S> =
                Box::new(move |state, parameters_values| Box::pin(func(state, parameters_values)));
            builder.routes_functions.3.push(func);
            builder.index_array.push(FuncIndex::new(
                FuncType::SerialStateful,
                builder.routes_functions.3.len() - 1,
            ));
            builder
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
        self.route_func_manager(route, ResponseKind::Info, move |mut builder| {
            let func: InfoFn = Box::new(move |parameters_values| Box::pin(func(parameters_values)));
            builder.routes_functions.4.push(func);
            builder.index_array.push(FuncIndex::new(
                FuncType::InfoStateless,
                builder.routes_functions.4.len() - 1,
            ));
            builder
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
        self.route_func_manager(route, ResponseKind::Info, move |mut builder| {
            let func: InfoStateFn<S> =
                Box::new(move |state, parameters_values| Box::pin(func(state, parameters_values)));
            builder.routes_functions.5.push(func);
            builder.index_array.push(FuncIndex::new(
                FuncType::InfoStateful,
                builder.routes_functions.5.len() - 1,
            ));
            builder
        })
    }

    /// Builds a [`Device`].
    ///
    /// **This method consumes the builder.**
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
            .remove_prohibited_hazards(self.allowed_hazards)
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
}
