use alloc::vec::Vec;

use tosca::device::DeviceData;
use tosca::events::EventsDescription;
use tosca::route::RouteConfigs;

use crate::response::Response;
use crate::server::{FuncIndex, Functions};
use crate::state::{State, ValueFromRef};

/// A generic `tosca` device.
///
/// A [`Device`] can only be passed to a [`crate::server::Server`].
pub struct Device<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    pub(crate) wifi_mac: [u8; 6],
    pub(crate) state: State<S>,
    pub(crate) description: DeviceData,
    pub(crate) main_route: &'static str,
    pub(crate) routes_functions: Functions<S>,
    pub(crate) index_array: Vec<FuncIndex>,
}

impl<S> Device<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    #[inline]
    pub(crate) fn new(
        wifi_mac: [u8; 6],
        state: State<S>,
        description: DeviceData,
        main_route: &'static str,
        routes_functions: Functions<S>,
        index_array: Vec<FuncIndex>,
    ) -> Self {
        Self {
            wifi_mac,
            state,
            description,
            main_route,
            routes_functions,
            index_array,
        }
    }

    #[inline]
    pub(crate) fn events_description(mut self, events_description: EventsDescription) -> Self {
        self.description = self.description.events_description(events_description);
        self
    }

    #[inline]
    pub(crate) fn into_internal(mut self) -> InternalDevice<S> {
        self.description.wifi_mac = Some(self.wifi_mac);
        InternalDevice {
            state: self.state,
            main_route: self.main_route,
            main_route_response: Response::json(&self.description),
            routes_functions: self.routes_functions,
            index_array: self.index_array,
            route_configs: self.description.route_configs,
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
