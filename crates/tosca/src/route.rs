use alloc::borrow::Cow;

use hashbrown::DefaultHashBuilder;

use indexmap::set::{IndexSet, IntoIter, Iter};

use log::error;

use serde::Serialize;

use crate::hazards::{Hazard, Hazards};
use crate::parameters::{Parameters, ParametersData};
use crate::response::ResponseKind;

use crate::macros::{mandatory_route, set};

/// The kind of `REST` request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub enum RestKind {
    /// `GET` request.
    Get,
    /// `PUT` request.
    Put,
    /// `POST` request.
    Post,
    /// `DELETE` request.
    Delete,
}

impl core::fmt::Display for RestKind {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Get => "GET",
            Self::Put => "PUT",
            Self::Post => "POST",
            Self::Delete => "DELETE",
        }
        .fmt(f)
    }
}

/// Route data.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub struct RouteData {
    /// Name.
    pub name: Cow<'static, str>,
    /// Path.
    pub path: Cow<'static, str>,
    /// Description.
    pub description: Option<Cow<'static, str>>,
    /// Hazards data.
    #[serde(skip_serializing_if = "Hazards::is_empty")]
    #[serde(default = "Hazards::new")]
    pub hazards: Hazards,
    /// Route parameters.
    #[serde(skip_serializing_if = "ParametersData::is_empty")]
    #[serde(default = "ParametersData::new")]
    pub parameters: ParametersData,
}

impl PartialEq for RouteData {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
    }
}

impl RouteData {
    fn new(route: Route) -> Self {
        Self {
            name: route.name.into(),
            path: route.path.into(),
            description: route.description.map(core::convert::Into::into),
            hazards: route.hazards,
            parameters: route.parameters.serialize_data(),
        }
    }
}

/// A route configuration.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub struct RouteConfig {
    /// Route data.
    #[serde(flatten)]
    pub data: RouteData,
    /// The kind of `REST` request.
    #[serde(rename = "REST kind")]
    pub rest_kind: RestKind,
    /// Response kind.
    #[serde(rename = "response kind")]
    pub response_kind: ResponseKind,
}

impl PartialEq for RouteConfig {
    fn eq(&self, other: &Self) -> bool {
        self.data.eq(&other.data) && self.rest_kind == other.rest_kind
    }
}

impl Eq for RouteConfig {}

impl core::hash::Hash for RouteConfig {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.data.path.hash(state);
        self.rest_kind.hash(state);
    }
}

impl RouteConfig {
    /// Changes the response kind.
    #[must_use]
    pub const fn change_response_kind(mut self, response_kind: ResponseKind) -> Self {
        self.response_kind = response_kind;
        self
    }

    fn new(route: Route) -> Self {
        Self {
            rest_kind: route.rest_kind,
            response_kind: ResponseKind::default(),
            data: RouteData::new(route),
        }
    }
}

set! {
  /// A collection of [`RouteConfig`]s.
  #[derive(Debug, Clone, PartialEq, Serialize)]
  #[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
  pub struct RouteConfigs(IndexSet<RouteConfig, DefaultHashBuilder>);
}

impl RouteConfigs {
    /// Merges the given [`RouteConfigs`] with the current one.
    #[must_use]
    #[inline]
    pub fn merge(mut self, other: Self) -> Self {
        self.0.extend(other);
        self
    }
}

/// A route definition.
///
/// Identifies a specific `REST` route that runs a task on a device when
/// invoked.
#[derive(Debug)]
pub struct Route {
    // Name.
    name: &'static str,
    // Path.
    path: &'static str,
    // REST kind.
    rest_kind: RestKind,
    // Description.
    description: Option<&'static str>,
    // Input route parameters.
    parameters: Parameters,
    // Hazards.
    hazards: Hazards,
}

impl PartialEq for Route {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.rest_kind == other.rest_kind
    }
}

impl Eq for Route {}

impl core::hash::Hash for Route {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state);
        self.rest_kind.hash(state);
    }
}

impl Route {
    /// Creates a [`Route`] through a `GET` API.
    #[must_use]
    #[inline]
    pub fn get(name: &'static str, path: &'static str) -> Self {
        Self::init(RestKind::Get, name, path)
    }

    /// Creates a [`Route`] through a `PUT` API.
    #[must_use]
    #[inline]
    pub fn put(name: &'static str, path: &'static str) -> Self {
        Self::init(RestKind::Put, name, path)
    }

    /// Creates a [`Route`] through a `POST` API.
    #[must_use]
    #[inline]
    pub fn post(name: &'static str, path: &'static str) -> Self {
        Self::init(RestKind::Post, name, path)
    }

    /// Creates a [`Route`] through a `DELETE` API.
    #[must_use]
    #[inline]
    pub fn delete(name: &'static str, path: &'static str) -> Self {
        Self::init(RestKind::Delete, name, path)
    }

    /// Sets the route description.
    #[must_use]
    pub const fn description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }

    /// Changes the route name.
    #[must_use]
    pub const fn change_name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// Changes the route path.
    #[must_use]
    pub const fn change_path(mut self, path: &'static str) -> Self {
        self.path = path;
        self
    }

    /// Adds [`Hazards`] to a [`Route`].
    #[must_use]
    #[inline]
    pub fn with_hazards(mut self, hazards: Hazards) -> Self {
        self.hazards = hazards;
        self
    }

    /// Adds an [`Hazard`] to a [`Route`].
    #[must_use]
    #[inline]
    pub fn with_hazard(mut self, hazard: Hazard) -> Self {
        self.hazards = Hazards::init(hazard);
        self
    }

    /// Adds an array of [`Hazard`]s to a [`Route`].
    #[must_use]
    #[inline]
    pub fn with_array_of_hazards<const N: usize>(mut self, hazards: [Hazard; N]) -> Self {
        self.hazards = Hazards::init_from_hazards(hazards);
        self
    }

    /// Adds [`Parameters`] to a [`Route`].
    #[must_use]
    #[inline]
    pub fn with_parameters(mut self, parameters: Parameters) -> Self {
        self.parameters = parameters;
        self
    }

    /// Returns the route path.
    #[must_use]
    pub const fn route(&self) -> &str {
        self.path
    }

    /// Returns the [`RestKind`].
    #[must_use]
    pub const fn kind(&self) -> RestKind {
        self.rest_kind
    }

    /// Returns [`Hazards`].
    #[must_use]
    pub const fn hazards(&self) -> &Hazards {
        &self.hazards
    }

    /// Returns [`Parameters`].
    #[must_use]
    pub const fn parameters(&self) -> &Parameters {
        &self.parameters
    }

    /// Removes prohibited [`Hazard`]s returning an updated [`Route`].
    #[must_use]
    #[inline]
    pub fn remove_prohibited_hazards(mut self, allowed_hazards: &[Hazard]) -> Self {
        let mut hazards = Hazards::new();
        for hazard in self.hazards {
            if allowed_hazards.contains(&hazard) {
                hazards.add(hazard);
            } else {
                error!("Hazards not allowed, removed: {hazard}");
            }
        }
        self.hazards = hazards;
        self
    }

    /// Serializes [`Route`] data.
    ///
    /// **It consumes the route.**
    #[must_use]
    #[inline]
    pub fn serialize_data(self) -> RouteConfig {
        RouteConfig::new(self)
    }

    fn init(rest_kind: RestKind, name: &'static str, path: &'static str) -> Self {
        Self {
            name,
            path,
            rest_kind,
            description: None,
            hazards: Hazards::new(),
            parameters: Parameters::new(),
        }
    }
}

set! {
  /// A collection of [`Route`]s.
  #[derive(Debug)]
  pub struct Routes(IndexSet<Route, DefaultHashBuilder>);
}

mandatory_route!(LightOnRoute, "/on", methods: [post, put]);
mandatory_route!(LightOffRoute, "/off", methods: [post, put]);

#[cfg(test)]
#[cfg(feature = "deserialize")]
mod tests {
    use crate::hazards::{Hazard, Hazards};
    use crate::parameters::{ParameterKind, Parameters, ParametersData};
    use crate::response::ResponseKind;
    use crate::{deserialize, serialize};

    use super::{RestKind, Route, RouteConfig, RouteData};

    fn route_config_empty(rest_kind: RestKind, desc: &'static str) -> RouteConfig {
        route_config_hazards(rest_kind, Hazards::new(), desc)
    }

    fn route_config_hazards(
        rest_kind: RestKind,
        hazards: Hazards,
        desc: &'static str,
    ) -> RouteConfig {
        route_config_parameters(rest_kind, hazards, desc, ParametersData::new())
    }

    fn route_config_parameters(
        rest_kind: RestKind,
        hazards: Hazards,
        desc: &'static str,
        parameters: ParametersData,
    ) -> RouteConfig {
        RouteConfig {
            rest_kind,
            response_kind: ResponseKind::default(),
            data: RouteData {
                name: "Route".into(),
                path: "/route".into(),
                description: Some(desc.into()),
                hazards,
                parameters,
            },
        }
    }

    #[test]
    fn test_all_routes() {
        assert_eq!(
            deserialize::<RouteConfig>(serialize(
                Route::get("Route", "/route")
                    .description("A GET route")
                    .serialize_data()
            )),
            route_config_empty(RestKind::Get, "A GET route",)
        );

        assert_eq!(
            deserialize::<RouteConfig>(serialize(
                Route::put("Route", "/route")
                    .description("A PUT route")
                    .serialize_data()
            )),
            route_config_empty(RestKind::Put, "A PUT route",)
        );

        assert_eq!(
            deserialize::<RouteConfig>(serialize(
                Route::post("Route", "/route")
                    .description("A POST route")
                    .serialize_data()
            )),
            route_config_empty(RestKind::Post, "A POST route",)
        );

        assert_eq!(
            deserialize::<RouteConfig>(serialize(
                Route::delete("Route", "/route")
                    .description("A DELETE route")
                    .serialize_data()
            )),
            route_config_empty(RestKind::Delete, "A DELETE route",)
        );
    }

    #[test]
    fn test_all_hazards() {
        assert_eq!(
            deserialize::<RouteConfig>(serialize(
                Route::get("Route", "/route")
                    .description("A GET route")
                    .with_hazard(Hazard::FireHazard)
                    .serialize_data()
            )),
            route_config_hazards(
                RestKind::Get,
                Hazards::new().insert(Hazard::FireHazard),
                "A GET route"
            )
        );

        assert_eq!(
            deserialize::<RouteConfig>(serialize(
                Route::get("Route", "/route")
                    .description("A GET route")
                    .with_hazards(
                        Hazards::new()
                            .insert(Hazard::FireHazard)
                            .insert(Hazard::AirPoisoning)
                    )
                    .serialize_data()
            )),
            route_config_hazards(
                RestKind::Get,
                Hazards::new()
                    .insert(Hazard::FireHazard)
                    .insert(Hazard::AirPoisoning),
                "A GET route"
            )
        );

        assert_eq!(
            deserialize::<RouteConfig>(serialize(
                Route::get("Route", "/route")
                    .description("A GET route")
                    .with_array_of_hazards([Hazard::FireHazard, Hazard::AirPoisoning])
                    .serialize_data()
            )),
            route_config_hazards(
                RestKind::Get,
                Hazards::new()
                    .insert(Hazard::FireHazard)
                    .insert(Hazard::AirPoisoning),
                "A GET route"
            )
        );
    }

    #[test]
    fn test_all_parameters() {
        let expected = route_config_parameters(
            RestKind::Get,
            Hazards::new(),
            "A GET route",
            ParametersData::new().insert(
                "rangeu64".into(),
                ParameterKind::RangeU64 {
                    min: 0,
                    max: 20,
                    step: 1,
                    default: 5,
                },
            ),
        );

        assert_eq!(
            deserialize::<RouteConfig>(serialize(
                Route::get("Route", "/route")
                    .description("A GET route")
                    .with_parameters(
                        Parameters::new()
                            .rangeu64_with_default("rangeu64", (0, 20, 1), 5)
                            .rangef64("rangef64", (0., 20., 0.1))
                    )
                    .serialize_data()
            )),
            expected
        );
    }
}

#[cfg(test)]
#[cfg(not(feature = "deserialize"))]
mod tests {
    use crate::route::{Hazard, Hazards};

    use super::Route;

    #[test]
    fn test_allowed_hazards() {
        const ALLOWED_HAZARDS: &[Hazard] = &[Hazard::FireHazard, Hazard::ElectricEnergyConsumption];

        // Wrong AirPoisoning hazard.
        let route = Route::get("Route", "/route")
            .description("A GET route")
            .with_hazards(
                Hazards::new()
                    .insert(Hazard::FireHazard)
                    .insert(Hazard::AirPoisoning),
            );

        let expected_hazards = Hazards::init(Hazard::FireHazard);
        assert_eq!(
            route.remove_prohibited_hazards(ALLOWED_HAZARDS).hazards,
            expected_hazards
        );
    }
}
