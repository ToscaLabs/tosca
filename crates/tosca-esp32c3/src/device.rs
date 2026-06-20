use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::format;
use alloc::vec::Vec;

use tosca::device::{DeviceDescription, DeviceEnvironment, DeviceScheme};
use tosca::events::EventsDescription;
use tosca::response::ResponseKind;
use tosca::route::{Route, RouteConfigs};

use esp_radio::wifi::WifiDevice;

use log::error;

use crate::error::{Error, ErrorKind};
use crate::parameters::ParametersPayloads;
use crate::response::{ErrorResponse, InfoResponse, OkResponse, Response, SerialResponse};
use crate::server::{
    FuncIndex, FuncType, Functions, InfoFn, InfoStateFn, OkFn, OkStateFn, SerialFn, SerialStateFn,
};
use crate::state::{State, ValueFromRef};

// Default main route.
const MAIN_ROUTE: &str = "/device";

/// A generic `tosca` device.
///
/// A [`Device`] can only be passed to a [`crate::server::Server`].
pub struct Device<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    pub(crate) wifi_mac: [u8; 6],
    pub(crate) description: DeviceDescription,
    pub(crate) main_route: &'static str,
    pub(crate) routes_functions: Functions<S>,
    pub(crate) index_array: Vec<FuncIndex>,
    pub(crate) state: State<S>,
}

impl Device<()> {
    /// Creates a [`Device`] without a state.
    #[must_use]
    #[inline]
    pub fn new(wifi_interface: &WifiDevice<'_>, scheme: DeviceScheme) -> Self {
        Self::with_state(wifi_interface, scheme, ())
    }
}

impl<S> Device<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    /// Creates a [`Device`] with the given state.
    #[must_use]
    #[inline]
    pub fn with_state(wifi_interface: &WifiDevice<'_>, scheme: DeviceScheme, state: S) -> Self {
        Self::init(wifi_interface, scheme, state)
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

    /// Builds a [`DeviceVerified`].
    ///
    /// # Errors
    ///
    /// - Returns an error if mandatory routes are missing or invalid.
    #[inline]
    pub fn build(mut self) -> crate::error::Result<DeviceVerified<S>> {
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
            error!("{message}");
            return Err(Error::new(ErrorKind::MandatoryRoutes, message));
        }

        self.description.data.wifi_mac = Some(self.wifi_mac);
        Ok(DeviceVerified(self))
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
            .remove_prohibited_hazards(self.description.data.scheme.allowed_hazards())
            .serialize_data()
            .change_response_kind(response_kind);

        if self.description.route_configs.contains(&route_config) {
            error!(
                "The route with prefix `{}` already exists!",
                route_config.data.path
            );
            return self;
        }

        self.description.route_configs.add(route_config);

        add_async_function(self)
    }

    #[inline]
    fn init(wifi_interface: &WifiDevice<'_>, device_scheme: DeviceScheme, state: S) -> Self {
        let wifi_mac = wifi_interface.mac_address();

        let description = DeviceDescription::new(device_scheme, MAIN_ROUTE, RouteConfigs::new())
            .environment(DeviceEnvironment::Embedded);

        Self {
            wifi_mac,
            description,
            main_route: MAIN_ROUTE,
            routes_functions: (
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ),
            index_array: Vec::new(),
            state: State(state),
        }
    }
}

/// A device with fully validated data.
///
/// [`DeviceVerified`] is a transient type, intended solely for passing
/// validated device data to [`crate::server::Server`].
pub struct DeviceVerified<S>(pub(crate) Device<S>)
where
    S: ValueFromRef + Send + Sync + 'static;

impl<S> DeviceVerified<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    #[inline]
    pub(crate) fn events_description(mut self, events_description: EventsDescription) -> Self {
        self.0.description = self.0.description.events_description(events_description);
        self
    }

    #[inline]
    pub(crate) fn into_internal(mut self) -> InternalDevice<S> {
        self.0.description.data.wifi_mac = Some(self.0.wifi_mac);
        InternalDevice {
            state: self.0.state,
            main_route: self.0.main_route,
            main_route_response: Response::json(&self.0.description),
            routes_functions: self.0.routes_functions,
            index_array: self.0.index_array,
            route_configs: self.0.description.route_configs,
        }
    }
}

pub(crate) struct InternalDevice<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    pub(crate) state: State<S>,
    pub(crate) main_route: &'static str,
    pub(crate) main_route_response: Response,
    pub(crate) routes_functions: Functions<S>,
    pub(crate) index_array: Vec<FuncIndex>,
    pub(crate) route_configs: RouteConfigs,
}
