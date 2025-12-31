use std::collections::HashMap;
use std::fmt::Write;
use std::future::Future;

use serde::Serialize;

use tracing::error;

use tosca::device::DeviceEnvironment;
use tosca::hazards::Hazards;
use tosca::parameters::{ParameterValue, ParametersData, ParametersValues};
use tosca::response::{ResponseKind, SERIALIZATION_ERROR};
use tosca::route::{RestKind, RouteConfig, RouteConfigs};

use crate::error::{Error, ErrorKind};
use crate::response::{InfoResponseParser, OkResponseParser, Response, SerialResponseParser};

fn slash_end(s: &str) -> &str {
    if s.len() > 1 && s.ends_with('/') {
        &s[..s.len() - 1]
    } else {
        s
    }
}

fn slash_start(s: &str) -> &str {
    if s.len() > 1 && s.starts_with('/') {
        &s[1..]
    } else {
        s
    }
}

fn slash_start_end(s: &str) -> &str {
    slash_start(slash_end(s))
}

fn compare_values_with_params_data(
    parameter_values: &ParametersValues,
    parameters_data: &ParametersData,
) -> Result<(), Error> {
    for (name, parameter_value) in parameter_values {
        let Some(parameter_kind) = parameters_data.get(name) else {
            return Err(parameter_error(format!("`{name}` does not exist")));
        };

        if !parameter_value.match_kind(parameter_kind) {
            return Err(parameter_error(format!(
                "Found type `{}` for `{name}`, expected type `{}`",
                parameter_value.as_type(),
                parameter_kind.as_type(),
            )));
        }
    }
    Ok(())
}

fn parameter_error(message: String) -> Error {
    error!(message);
    Error::new(ErrorKind::WrongParameter, message)
}

#[derive(Debug, PartialEq)]
struct RequestData {
    request: String,
    parameters: HashMap<String, String>,
}

impl RequestData {
    const fn new(request: String, parameters: HashMap<String, String>) -> Self {
        Self {
            request,
            parameters,
        }
    }
}

pub(crate) fn create_requests(
    route_configs: RouteConfigs,
    complete_address: &str,
    main_route: &str,
    environment: DeviceEnvironment,
) -> HashMap<String, Request> {
    route_configs
        .into_iter()
        .map(|route| {
            (
                route.data.path.to_string(),
                Request::new(complete_address, main_route, environment, route),
            )
        })
        .collect()
}

/// Request information.
///
/// Each request is identified by the data of its associated route and the
/// corresponding response.
pub struct RequestInfo<'device> {
    /// Route name.
    pub route: &'device str,
    /// Route description.
    pub description: Option<&'device str>,
    /// Rest kind.
    pub rest_kind: RestKind,
    /// Route hazards.
    pub hazards: &'device Hazards,
    /// Parameters data.
    ///
    /// If the request has no parameters, the reference will be empty.
    pub parameters_data: &'device ParametersData,
    /// Response kind.
    pub response_kind: ResponseKind,
}

impl<'device> RequestInfo<'device> {
    pub(crate) fn new(route: &'device str, request: &'device Request) -> Self {
        Self {
            route,
            description: request.description.as_deref(),
            rest_kind: request.kind,
            hazards: &request.hazards,
            parameters_data: &request.parameters_data,
            response_kind: request.response_kind,
        }
    }
}

/// A device request to be sent to a device.
///
/// A request can either be plain, with no associated parameters, or include
/// parameters that serve as inputs for device tasks.
#[derive(Debug, PartialEq, Serialize)]
pub struct Request {
    pub(crate) kind: RestKind,
    pub(crate) hazards: Hazards,
    pub(crate) route: String,
    pub(crate) description: Option<String>,
    pub(crate) parameters_data: ParametersData,
    pub(crate) response_kind: ResponseKind,
    pub(crate) device_environment: DeviceEnvironment,
}

impl Request {
    /// Returns an immutable reference to the request [`Hazards`].
    #[must_use]
    pub fn hazards(&self) -> &Hazards {
        &self.hazards
    }

    /// Returns a request [`RestKind`].
    #[must_use]
    pub fn kind(&self) -> RestKind {
        self.kind
    }

    /// Returns an immutable reference to the [`ParametersData`] associated with
    /// a request.
    ///
    /// If [`None`], the request **does not** contain any [`ParametersData`].
    #[must_use]
    pub fn parameters_data(&self) -> Option<&ParametersData> {
        self.parameters_data
            .is_empty()
            .then_some(&self.parameters_data)
    }

    pub(crate) fn new(
        address: &str,
        main_route: &str,
        device_environment: DeviceEnvironment,
        route_config: RouteConfig,
    ) -> Self {
        let kind = route_config.rest_kind;
        let route = format!(
            "{}/{}/{}",
            slash_end(address),
            slash_start_end(main_route),
            slash_start_end(&route_config.data.path)
        );
        let hazards = route_config.data.hazards;
        let parameters_data = route_config.data.parameters;
        let response_kind = route_config.response_kind;

        Self {
            kind,
            hazards,
            route,
            description: route_config.data.description.map(|s| s.to_string()),
            parameters_data,
            response_kind,
            device_environment,
        }
    }

    pub(crate) async fn retrieve_response<F, Fut>(
        &self,
        skip: bool,
        retrieve_response: F,
    ) -> Result<Response, Error>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<reqwest::Response, Error>>,
    {
        if skip {
            return Ok(Response::Skipped);
        }

        let response = retrieve_response().await?;

        Ok(match self.response_kind {
            ResponseKind::Ok => Response::OkBody(OkResponseParser::new(response)),
            ResponseKind::Serial => Response::SerialBody(SerialResponseParser::new(response)),
            ResponseKind::Info => Response::InfoBody(InfoResponseParser::new(response)),
            #[cfg(feature = "stream")]
            ResponseKind::Stream => {
                Response::StreamBody(crate::response::StreamResponse::new(response))
            }
        })
    }

    pub(crate) async fn plain_send(&self) -> Result<reqwest::Response, Error> {
        let request_data =
            self.request_data(|| self.axum_get_plain(), || self.create_params_plain());

        self.parameters_send(request_data).await
    }

    pub(crate) async fn create_response(
        &self,
        parameters: &ParametersValues<'_>,
    ) -> Result<reqwest::Response, Error> {
        let request_data = self.create_request(parameters)?;
        self.parameters_send(request_data).await
    }

    async fn parameters_send(&self, request_data: RequestData) -> Result<reqwest::Response, Error> {
        let RequestData {
            request,
            parameters,
        } = request_data;

        let client = reqwest::Client::new();

        let request_builder = match self.kind {
            RestKind::Get => client.get(request),
            RestKind::Post => client.post(request),
            RestKind::Put => client.put(request),
            RestKind::Delete => client.delete(request),
        };

        let request_builder = if self.kind != RestKind::Get && !parameters.is_empty() {
            request_builder.json(&parameters)
        } else {
            request_builder
        };

        // Close the connection after issuing a request.
        let response = request_builder.header("Connection", "close").send().await?;

        // TODO: Analyze the response status.
        // A 404 status (route not found) might be returned when a
        // device is down or in case of a malformed route.
        // A 405 status (method not allowed) in case of a wrong REST method.
        // If the status is 200, the device response is valid. If the
        // response is 500, an error occurred during a device operation.
        //
        //  Add a logger to record the response. In this way, we do not block
        //  the process returning an error. An app using the controller as
        //  dependency can then consult the logger to obtain the internal
        //  problem.

        // Checks whether serialization errors have occurred on the device.
        // If the serialization error header is present, the response
        // is considered invalid.
        // Additionally, the response is invalid if the body
        // cannot be converted to a string.
        if response.headers().contains_key(SERIALIZATION_ERROR) {
            match response.text().await {
                Ok(serial_error) => {
                    error!("Serialization error encountered on the device side: {serial_error}");
                    return Err(Error::new(ErrorKind::Request, serial_error));
                }
                Err(err) => {
                    error!("Error occurred while converting the request into text: {err}");
                    return Err(Error::new(ErrorKind::Request, err.to_string()));
                }
            }
        }

        Ok(response)
    }

    fn request_data<A, F>(&self, axum_get: A, params: F) -> RequestData
    where
        A: FnOnce() -> String,
        F: FnOnce() -> HashMap<String, String>,
    {
        let request =
            if self.kind == RestKind::Get && self.device_environment == DeviceEnvironment::Os {
                axum_get()
            } else {
                self.route.clone()
            };

        let parameters = params();

        RequestData::new(request, parameters)
    }

    fn create_request(&self, parameters: &ParametersValues) -> Result<RequestData, Error> {
        // Compare parameters values with parameters data.
        compare_values_with_params_data(parameters, &self.parameters_data)?;

        Ok(self.request_data(
            || self.axum_get(parameters),
            || self.create_params(parameters),
        ))
    }

    fn axum_get_plain(&self) -> String {
        let mut route = self.route.clone();
        for (_, parameter_kind) in &self.parameters_data {
            // TODO: Consider returning `Option<String>`
            if let Err(e) = write!(
                route,
                "/{}",
                ParameterValue::from_parameter_kind(parameter_kind)
            ) {
                error!("Error in adding a path to a route : {e}");
                break;
            }
        }
        route
    }

    fn create_params_plain(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        for (name, parameter_kind) in &self.parameters_data {
            params.insert(
                name.clone(),
                format!("{}", ParameterValue::from_parameter_kind(parameter_kind)),
            );
        }
        params
    }

    // Axum parameters: hello/{{1}}/{{2}}
    //                  hello/0.5/1
    fn axum_get(&self, parameters: &ParametersValues) -> String {
        let mut route = String::from(&self.route);
        for (name, parameter_kind) in &self.parameters_data {
            let value = if let Some(value) = parameters.get(name) {
                format!("{value}")
            } else {
                format!("{}", ParameterValue::from_parameter_kind(parameter_kind))
            };
            // TODO: Consider returning `Option<String>`
            if let Err(e) = write!(route, "/{value}") {
                error!("Error in adding a path to a route : {e}");
                break;
            }
        }

        route
    }

    fn create_params(&self, parameters: &ParametersValues<'_>) -> HashMap<String, String> {
        let mut params = HashMap::new();
        for (name, parameter_kind) in &self.parameters_data {
            let (name, value) = if let Some(value) = parameters.get(name) {
                (name, format!("{value}"))
            } else {
                (
                    name,
                    format!("{}", ParameterValue::from_parameter_kind(parameter_kind)),
                )
            };
            params.insert(name.clone(), value);
        }
        params
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tosca::device::DeviceEnvironment;
    use tosca::hazards::{Hazard, Hazards};
    use tosca::parameters::{ParameterKind, Parameters, ParametersData, ParametersValues};
    use tosca::route::{RestKind, Route, RouteConfig};

    use super::{Request, RequestData, ResponseKind, parameter_error};

    const ADDRESS_ROUTE: &str = "http://tosca.local/";
    const ADDRESS_ROUTE_WITHOUT_SLASH: &str = "http://tosca.local/";
    const COMPLETE_ROUTE: &str = "http://tosca.local/light/route";

    fn plain_request(route: Route, kind: RestKind, hazards: Hazards) {
        let route = route.serialize_data();
        let description = route
            .data
            .description
            .as_ref()
            .map(std::string::ToString::to_string);

        let request = Request::new(ADDRESS_ROUTE, "light/", DeviceEnvironment::Os, route);

        assert_eq!(
            request,
            Request {
                kind,
                hazards,
                route: COMPLETE_ROUTE.into(),
                description,
                parameters_data: ParametersData::new(),
                response_kind: ResponseKind::Ok,
                device_environment: DeviceEnvironment::Os,
            }
        );
    }

    fn request_with_parameters(route: Route, kind: RestKind, hazards: &Hazards) {
        let route = route
            .with_parameters(
                Parameters::new()
                    .rangeu64_with_default("rangeu64", (0, 20, 1), 5)
                    .rangef64("rangef64", (0., 20., 0.1)),
            )
            .serialize_data();
        let description = route
            .data
            .description
            .as_ref()
            .map(std::string::ToString::to_string);

        let parameters_data = ParametersData::new()
            .insert(
                "rangeu64".into(),
                ParameterKind::RangeU64 {
                    min: 0,
                    max: 20,
                    step: 1,
                    default: 5,
                },
            )
            .insert(
                "rangef64".into(),
                ParameterKind::RangeF64 {
                    min: 0.,
                    max: 20.,
                    step: 0.1,
                    default: 0.,
                },
            );

        let request = Request::new(ADDRESS_ROUTE, "light/", DeviceEnvironment::Os, route);

        assert_eq!(
            request,
            Request {
                kind,
                hazards: hazards.clone(),
                route: COMPLETE_ROUTE.into(),
                description,
                parameters_data,
                response_kind: ResponseKind::Ok,
                device_environment: DeviceEnvironment::Os,
            }
        );

        // Non-existent parameter.
        assert_eq!(
            request.create_request(ParametersValues::new().u64("wrong", 0)),
            Err(parameter_error("`wrong` does not exist".into()))
        );

        // Wrong parameter type.
        assert_eq!(
            request.create_request(ParametersValues::new().f64("rangeu64", 0.)),
            Err(parameter_error(
                "Found type `f64` for `rangeu64`, expected type `u64`".into()
            ))
        );

        let mut parameters = HashMap::with_capacity(2);
        parameters.insert("rangeu64".into(), "3".into());
        parameters.insert("rangef64".into(), "0".into());

        assert_eq!(
            request.create_request(ParametersValues::new().u64("rangeu64", 3)),
            Ok(RequestData {
                request: if kind == RestKind::Get {
                    format!("{COMPLETE_ROUTE}/3/0")
                } else {
                    COMPLETE_ROUTE.into()
                },
                parameters,
            })
        );
    }

    fn request_builder(
        route: &str,
        main_route: &str,
        device_environment: DeviceEnvironment,
        route_config: RouteConfig,
    ) {
        assert_eq!(
            Request::new(route, main_route, device_environment, route_config),
            Request {
                kind: RestKind::Put,
                hazards: Hazards::new(),
                route: COMPLETE_ROUTE.into(),
                description: None,
                parameters_data: ParametersData::new(),
                response_kind: ResponseKind::Ok,
                device_environment: DeviceEnvironment::Os,
            }
        );
    }

    #[test]
    fn check_request_builder() {
        let route = Route::put("Route", "/route").serialize_data();
        let environment = DeviceEnvironment::Os;

        request_builder(ADDRESS_ROUTE, "light/", environment, route.clone());
        request_builder(ADDRESS_ROUTE_WITHOUT_SLASH, "light", environment, route);
    }

    #[test]
    fn create_plain_get_request() {
        let route = Route::get("Route", "/route").description("A GET route.");
        plain_request(route, RestKind::Get, Hazards::new());
    }

    #[test]
    fn create_plain_post_request() {
        let route = Route::post("Route", "/route").description("A POST route.");
        plain_request(route, RestKind::Post, Hazards::new());
    }

    #[test]
    fn create_plain_put_request() {
        let route = Route::put("Route", "/route").description("A PUT route.");
        plain_request(route, RestKind::Put, Hazards::new());
    }

    #[test]
    fn create_plain_delete_request() {
        let route = Route::delete("Route", "/route").description("A DELETE route.");
        plain_request(route, RestKind::Delete, Hazards::new());
    }

    #[test]
    fn create_plain_get_request_with_hazards() {
        let hazards = Hazards::new()
            .insert(Hazard::FireHazard)
            .insert(Hazard::AirPoisoning);
        plain_request(
            Route::get("Route", "/route")
                .description("A GET route.")
                .with_hazards(hazards.clone()),
            RestKind::Get,
            hazards,
        );
    }

    #[test]
    fn create_get_request_with_parameters() {
        request_with_parameters(
            Route::get("Route", "/route").description("A GET route."),
            RestKind::Get,
            &Hazards::new(),
        );
    }

    #[test]
    fn create_post_request_with_parameters() {
        let route = Route::post("Route", "/route").description("A POST route.");
        request_with_parameters(route, RestKind::Post, &Hazards::new());
    }

    #[test]
    fn create_put_request_with_parameters() {
        let route = Route::put("Route", "/route").description("A PUT route.");
        request_with_parameters(route, RestKind::Put, &Hazards::new());
    }

    #[test]
    fn create_delete_request_with_parameters() {
        let route = Route::delete("Route", "/route").description("A DELETE route.");
        request_with_parameters(route, RestKind::Delete, &Hazards::new());
    }

    #[test]
    fn create_get_request_with_hazards_and_parameters() {
        let hazards = Hazards::new()
            .insert(Hazard::FireHazard)
            .insert(Hazard::AirPoisoning);

        request_with_parameters(
            Route::get("Route", "/route")
                .description("A GET route.")
                .with_hazards(hazards.clone()),
            RestKind::Get,
            &hazards,
        );
    }
}
