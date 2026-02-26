use axum::Router;

use tosca::device::DeviceKind;
use tosca::hazards::Hazard;
use tosca::route::{LightOffRoute, LightOnRoute, Route, RouteConfig};

use crate::device::Device;
use crate::error::Result;
use crate::responses::{BaseResponse, MandatoryResponse};

// Default main route.
const MAIN_ROUTE: &str = "/light";

// Allowed hazards.
const ALLOWED_HAZARDS: &[Hazard] = &[
    Hazard::FireHazard,
    Hazard::ElectricEnergyConsumption,
    Hazard::LogEnergyConsumption,
];

/// A `light` device.
///
/// Its methods guide in the definition of a correct light.
///
/// The default main route is **/light**.
pub struct Light<const M1: bool, const M2: bool, S = ()>
where
    S: Clone + Send + Sync + 'static,
{
    // Internal device.
    device: Device<S>,
    // Turn light on.
    turn_light_on: MandatoryResponse<M1>,
    // Turn light off.
    turn_light_off: MandatoryResponse<M2>,
}

impl Default for Light<false, false, ()> {
    fn default() -> Self {
        Self::new()
    }
}

impl Light<false, false, ()> {
    /// Creates a [`Light`] without a state.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self::with_state(())
    }
}

impl<S> Light<false, false, S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Creates a [`Light`] with a state.
    #[inline]
    pub fn with_state(state: S) -> Self {
        let device = Device::init(&DeviceKind::Light, state).main_route(MAIN_ROUTE);

        Self {
            device,
            turn_light_on: MandatoryResponse::empty(),
            turn_light_off: MandatoryResponse::empty(),
        }
    }

    /// Turns on the light.
    ///
    /// **Calling this method is required, or a compilation error will occur.**
    pub fn turn_light_on(
        self,
        route: LightOnRoute,
        turn_light_on: impl FnOnce(Route, S) -> MandatoryResponse<false>,
    ) -> Light<true, false, S> {
        let turn_light_on = turn_light_on(route.into_route(), self.device.state.clone());

        Light {
            device: self.device,
            turn_light_on: MandatoryResponse::init(turn_light_on.base_response),
            turn_light_off: self.turn_light_off,
        }
    }
}

impl<S> Light<true, false, S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Turns off the light.
    ///
    /// **Calling this method is required, or a compilation error will occur.**
    pub fn turn_light_off(
        self,
        route: LightOffRoute,
        turn_light_off: impl FnOnce(Route, S) -> MandatoryResponse<false>,
    ) -> Light<true, true, S> {
        let turn_light_off = turn_light_off(route.into_route(), self.device.state.clone());

        Light {
            device: self.device,
            turn_light_on: self.turn_light_on,
            turn_light_off: MandatoryResponse::init(turn_light_off.base_response),
        }
    }
}

impl<S> Light<true, true, S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Sets the main route.
    #[must_use]
    #[inline]
    pub fn main_route(mut self, main_route: &'static str) -> Self {
        self.device = self.device.main_route(main_route);
        self
    }

    /// Adds a route to [`Light`].
    ///
    /// # Errors
    ///
    /// Returns an error if the route contains hazards not allowed for
    /// [`Light`].
    pub fn route(mut self, light_route: impl FnOnce(S) -> BaseResponse) -> Result<Self> {
        let base_response = light_route(self.device.state.clone());

        self.device = self
            .device
            .response_data(Self::check_allowed_hazards(base_response));

        Ok(self)
    }

    /// Adds an informative route to [`Light`].
    #[must_use]
    pub fn info_route(mut self, light_info_route: impl FnOnce(S, ()) -> BaseResponse) -> Self {
        let base_response = light_info_route(self.device.state.clone(), ());

        self.device = self
            .device
            .response_data(Self::check_allowed_hazards(base_response));

        self
    }

    /// Builds a [`Device`].
    ///
    /// **This method consumes the light.**
    pub fn build(self) -> Device<S> {
        self.device.mandatory_response_data([
            Self::check_allowed_hazards(self.turn_light_on.base_response),
            Self::check_allowed_hazards(self.turn_light_off.base_response),
        ])
    }

    fn check_allowed_hazards(base_response: BaseResponse) -> (RouteConfig, Router) {
        base_response.finalize_with_hazards(ALLOWED_HAZARDS)
    }
}

#[cfg(test)]
mod tests {
    use tosca::hazards::Hazard;
    use tosca::parameters::Parameters;
    use tosca::route::Route;

    use axum::extract::{Json, State};

    use serde::{Deserialize, Serialize};

    use crate::devices::light::{LightOffRoute, LightOnRoute};
    use crate::responses::error::ErrorResponse;
    use crate::responses::ok::{
        OkResponse, mandatory_ok_stateful, mandatory_ok_stateless, ok_stateful, ok_stateless,
    };
    use crate::responses::serial::{
        SerialResponse, mandatory_serial_stateful, mandatory_serial_stateless, serial_stateful,
        serial_stateless,
    };

    use super::Light;

    #[derive(Clone)]
    struct LightState;

    #[derive(Deserialize)]
    struct Inputs {
        brightness: f64,
        #[serde(alias = "save-energy")]
        save_energy: bool,
    }

    #[derive(Serialize, Deserialize)]
    struct LightOnResponse {
        brightness: f64,
        #[serde(rename = "save-energy")]
        save_energy: bool,
    }

    async fn turn_light_on(
        State(_state): State<LightState>,
        Json(inputs): Json<Inputs>,
    ) -> Result<SerialResponse<LightOnResponse>, ErrorResponse> {
        Ok(SerialResponse::new(LightOnResponse {
            brightness: inputs.brightness,
            save_energy: inputs.save_energy,
        }))
    }

    async fn turn_light_on_stateless(
        Json(inputs): Json<Inputs>,
    ) -> Result<SerialResponse<LightOnResponse>, ErrorResponse> {
        Ok(SerialResponse::new(LightOnResponse {
            brightness: inputs.brightness,
            save_energy: inputs.save_energy,
        }))
    }

    async fn turn_light_off(State(_state): State<LightState>) -> Result<OkResponse, ErrorResponse> {
        Ok(OkResponse::ok())
    }

    async fn turn_light_off_stateless() -> Result<OkResponse, ErrorResponse> {
        Ok(OkResponse::ok())
    }

    async fn toggle(State(_state): State<LightState>) -> Result<OkResponse, ErrorResponse> {
        Ok(OkResponse::ok())
    }

    async fn toggle_stateless() -> Result<OkResponse, ErrorResponse> {
        Ok(OkResponse::ok())
    }

    struct Routes {
        light_on: LightOnRoute,
        light_on_post: Route,
        light_off: LightOffRoute,
        toggle: Route,
    }

    #[inline]
    fn create_routes() -> Routes {
        Routes {
            light_on: LightOnRoute::put("On")
                .description("Turn light on.")
                .with_hazard(Hazard::ElectricEnergyConsumption)
                .with_parameters(
                    Parameters::new()
                        .rangef64("brightness", (0., 20., 0.1))
                        .bool("save-energy", false),
                ),

            light_on_post: Route::post("On Post", "/on-post")
                .description("Turn light on.")
                .with_hazard(Hazard::ElectricEnergyConsumption)
                .with_parameters(
                    Parameters::new()
                        .rangef64("brightness", (0., 20., 0.1))
                        .bool("save-energy", false),
                ),

            light_off: LightOffRoute::put("Off").description("Turn light off."),

            toggle: Route::put("Toggle", "/toggle")
                .description("Toggle a light.")
                .with_hazard(Hazard::ElectricEnergyConsumption),
        }
    }

    #[test]
    fn complete_with_state() {
        let routes = create_routes();

        Light::with_state(LightState {})
            .turn_light_on(routes.light_on, mandatory_serial_stateful(turn_light_on))
            .turn_light_off(routes.light_off, mandatory_ok_stateful(turn_light_off))
            .route(serial_stateful(routes.light_on_post, turn_light_on))
            .unwrap()
            .route(ok_stateful(routes.toggle, toggle))
            .unwrap()
            .build();
    }

    #[test]
    fn without_response_with_state() {
        let routes = create_routes();

        Light::with_state(LightState {})
            .turn_light_on(routes.light_on, mandatory_serial_stateful(turn_light_on))
            .turn_light_off(routes.light_off, mandatory_ok_stateful(turn_light_off))
            .build();
    }

    #[test]
    fn stateless_response_with_state() {
        let routes = create_routes();

        Light::with_state(LightState {})
            .turn_light_on(routes.light_on, mandatory_serial_stateful(turn_light_on))
            .turn_light_off(routes.light_off, mandatory_ok_stateful(turn_light_off))
            .route(serial_stateful(routes.light_on_post, turn_light_on))
            .unwrap()
            .route(ok_stateless(routes.toggle, toggle_stateless))
            .unwrap()
            .build();
    }

    #[test]
    fn complete_without_state() {
        let routes = create_routes();

        Light::new()
            .turn_light_on(
                routes.light_on,
                mandatory_serial_stateless(turn_light_on_stateless),
            )
            .turn_light_off(
                routes.light_off,
                mandatory_ok_stateless(turn_light_off_stateless),
            )
            .route(serial_stateless(
                routes.light_on_post,
                turn_light_on_stateless,
            ))
            .unwrap()
            .route(ok_stateless(routes.toggle, toggle_stateless))
            .unwrap()
            .build();
    }

    #[test]
    fn without_response_and_state() {
        let routes = create_routes();

        Light::new()
            .turn_light_on(
                routes.light_on,
                mandatory_serial_stateless(turn_light_on_stateless),
            )
            .turn_light_off(
                routes.light_off,
                mandatory_ok_stateless(turn_light_off_stateless),
            )
            .build();
    }
}
