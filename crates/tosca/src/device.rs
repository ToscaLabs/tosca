use alloc::borrow::Cow;

use serde::Serialize;

use crate::economy::Economy;
use crate::energy::Energy;
use crate::events::EventsDescription;
use crate::hazards::Hazard;
use crate::route::RouteConfigs;

/// A device kind.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub enum DeviceKind {
    /// Light.
    Light,
    /// Custom.
    Custom(Cow<'static, str>),
}

impl core::fmt::Display for DeviceKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Light => "Light",
            Self::Custom(kind) => kind,
        })
    }
}

/// A device scheme.
///
/// A scheme consists of:
///
/// - A device kind
/// - A list of all mandatory route names
/// - A list of hazards allowed for the device
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DeviceScheme {
    pub(super) kind: DeviceKind,
    pub(super) mandatory_routes: &'static [&'static str],
    pub(super) allowed_hazards: &'static [Hazard],
}

impl DeviceScheme {
    /// Creates a base custom scheme with a device kind only.
    #[must_use]
    #[inline]
    pub fn base_custom_scheme(custom_kind: &'static str) -> Self {
        Self::new(DeviceKind::Custom(Cow::Borrowed(custom_kind)), &[], &[])
    }

    /// Creates a custom scheme with device kind and mandatory routes.
    #[must_use]
    #[inline]
    pub fn custom_scheme_with_mandatory_routes(
        custom_kind: &'static str,
        mandatory_routes: &'static [&'static str],
    ) -> Self {
        Self::new(
            DeviceKind::Custom(Cow::Borrowed(custom_kind)),
            mandatory_routes,
            &[],
        )
    }

    /// Creates a custom scheme with device kind and allowed hazards.
    #[must_use]
    #[inline]
    pub fn custom_scheme_with_allowed_hazards(
        custom_kind: &'static str,
        allowed_hazards: &'static [Hazard],
    ) -> Self {
        Self::new(
            DeviceKind::Custom(Cow::Borrowed(custom_kind)),
            &[],
            allowed_hazards,
        )
    }

    /// Creates a complete custom scheme.
    #[must_use]
    #[inline]
    pub fn custom_scheme(
        custom_kind: &'static str,
        mandatory_routes: &'static [&'static str],
        allowed_hazards: &'static [Hazard],
    ) -> Self {
        Self::new(
            DeviceKind::Custom(Cow::Borrowed(custom_kind)),
            mandatory_routes,
            allowed_hazards,
        )
    }

    /// Returns an immutable reference to [`DeviceKind`].
    #[must_use]
    pub const fn kind(&self) -> &DeviceKind {
        &self.kind
    }

    /// Returns an immutable reference to mandatory routes.
    #[must_use]
    pub const fn mandatory_routes(&self) -> &'static [&'static str] {
        self.mandatory_routes
    }

    /// Returns an immutable reference to allowed hazards.
    #[must_use]
    pub const fn allowed_hazards(&self) -> &'static [Hazard] {
        self.allowed_hazards
    }

    const fn new(
        kind: DeviceKind,
        mandatory_routes: &'static [&'static str],
        allowed_hazards: &'static [Hazard],
    ) -> Self {
        Self {
            kind,
            mandatory_routes,
            allowed_hazards,
        }
    }
}

/// A series of device schemes.
pub mod schemes {
    use crate::hazards::Hazard;

    use super::{DeviceKind, DeviceScheme};

    /// A light scheme.
    pub const LIGHT_SCHEME: DeviceScheme = DeviceScheme::new(
        DeviceKind::Light,
        &["/on", "/off"],
        &[
            Hazard::FireHazard,
            Hazard::ElectricEnergyConsumption,
            Hazard::LogEnergyConsumption,
            Hazard::LogUsageTime,
            Hazard::PowerOutage,
            Hazard::PowerSurge,
        ],
    );
}

/// An owned device scheme.
///
/// A scheme consists of:
///
/// - A device kind
/// - A list of all mandatory route names
/// - A list of hazards allowed for the device
#[derive(Debug, Clone, PartialEq, Serialize, serde::Deserialize)]
#[cfg(feature = "deserialize")]
pub struct DeviceSchemeOwned {
    kind: DeviceKind,
    mandatory_routes: alloc::vec::Vec<alloc::string::String>,
    allowed_hazards: alloc::vec::Vec<Hazard>,
}

#[cfg(feature = "deserialize")]
impl From<DeviceScheme> for DeviceSchemeOwned {
    fn from(device_scheme: DeviceScheme) -> Self {
        Self {
            kind: device_scheme.kind,
            mandatory_routes: device_scheme
                .mandatory_routes
                .iter()
                .copied()
                .map(alloc::string::String::from)
                .collect(),
            allowed_hazards: device_scheme.allowed_hazards.to_vec(),
        }
    }
}

#[cfg(feature = "deserialize")]
impl DeviceSchemeOwned {
    /// Returns an immutable reference to [`DeviceKind`].
    #[must_use]
    pub const fn kind(&self) -> &DeviceKind {
        &self.kind
    }

    /// Returns an immutable reference to mandatory routes.
    #[must_use]
    #[inline]
    pub fn mandatory_routes(&self) -> &[alloc::string::String] {
        &self.mandatory_routes
    }

    /// Returns an immutable reference to allowed hazards.
    #[must_use]
    #[inline]
    pub fn allowed_hazards(&self) -> &[Hazard] {
        &self.allowed_hazards
    }
}

/// Device environment.
///
/// Indicates the type of underlying hardware architecture,
/// operating or embedded system, allowing the controller to adjust its
/// operations accordingly.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub enum DeviceEnvironment {
    /// Embedded system.
    Embedded,
    /// Operating system.
    Os,
}

/// Device metrics.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub struct DeviceMetrics {
    /// Energy metrics.
    #[serde(skip_serializing_if = "Energy::is_empty")]
    #[serde(default = "Energy::empty")]
    pub energy: Energy,
    /// Economy metrics.
    #[serde(skip_serializing_if = "Economy::is_empty")]
    #[serde(default = "Economy::empty")]
    pub economy: Economy,
}

impl DeviceMetrics {
    /// Creates a [`DeviceMetrics`] populated exclusively with [`Energy`]
    /// metrics.
    #[must_use]
    pub const fn with_energy(energy: Energy) -> Self {
        Self {
            energy,
            economy: Economy::empty(),
        }
    }

    /// Creates a [`DeviceMetrics`] populated exclusively with [`Economy`]
    /// metrics.
    #[must_use]
    pub const fn with_economy(economy: Economy) -> Self {
        Self {
            energy: Energy::empty(),
            economy,
        }
    }

    /// Adds [`Energy`] data.
    #[inline]
    #[must_use]
    pub fn add_energy(mut self, energy: Energy) -> Self {
        self.energy = energy;
        self
    }

    /// Adds [`Economy`] data.
    #[inline]
    #[must_use]
    pub fn add_economy(mut self, economy: Economy) -> Self {
        self.economy = economy;
        self
    }
}

/// Device data.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub struct DeviceData {
    /// Device scheme.
    #[cfg(not(feature = "deserialize"))]
    pub scheme: DeviceScheme,
    /// Device scheme.
    #[cfg(feature = "deserialize")]
    pub scheme: DeviceSchemeOwned,
    /// Device environment.
    pub environment: DeviceEnvironment,
    /// Device description.
    pub description: Option<Cow<'static, str>>,
    /// Wi-Fi MAC address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wifi_mac: Option<[u8; 6]>,
    /// Ethernet MAC address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ethernet_mac: Option<[u8; 6]>,
}

impl DeviceData {
    #[inline]
    fn new(scheme: DeviceScheme) -> Self {
        Self {
            #[cfg(not(feature = "deserialize"))]
            scheme,
            #[cfg(feature = "deserialize")]
            scheme: DeviceSchemeOwned::from(scheme),
            environment: DeviceEnvironment::Embedded,
            description: None,
            wifi_mac: None,
            ethernet_mac: None,
        }
    }
}

/// Device description.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub struct DeviceDescription {
    /// Device data.
    #[serde(flatten)]
    pub data: DeviceData,
    /// Device main route.
    pub main_route: Cow<'static, str>,
    /// All device route configurations.
    pub route_configs: RouteConfigs,
    /// Events description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events_description: Option<EventsDescription>,
}

impl DeviceDescription {
    /// Creates [`DeviceDescription`].
    #[inline]
    #[must_use]
    pub fn new(
        scheme: DeviceScheme,
        main_route: &'static str,
        route_configs: RouteConfigs,
    ) -> Self {
        Self {
            data: DeviceData::new(scheme),
            main_route: Cow::Borrowed(main_route),
            route_configs,
            events_description: None,
        }
    }

    /// Sets the device environment.
    #[must_use]
    pub const fn environment(mut self, environment: DeviceEnvironment) -> Self {
        self.data.environment = environment;
        self
    }

    /// Sets the Wi-Fi MAC address.
    #[must_use]
    pub const fn wifi_mac(mut self, wifi_mac: [u8; 6]) -> Self {
        self.data.wifi_mac = Some(wifi_mac);
        self
    }

    /// Sets the Ethernet MAC address.
    #[must_use]
    pub const fn ethernet_mac(mut self, ethernet_mac: [u8; 6]) -> Self {
        self.data.ethernet_mac = Some(ethernet_mac);
        self
    }

    /// Sets the device text description.
    #[inline]
    #[must_use]
    pub fn text_description(mut self, description: &'static str) -> Self {
        self.data.description = Some(Cow::Borrowed(description));
        self
    }

    /// Adds an [`EventsDescription`].
    #[must_use]
    #[inline]
    pub fn events_description(mut self, events_description: EventsDescription) -> Self {
        self.events_description = Some(events_description);
        self
    }
}

#[cfg(test)]
#[cfg(not(feature = "deserialize"))]
mod tests {
    use alloc::borrow::Cow;

    use crate::hazards::Hazard;

    use super::{DeviceKind, DeviceScheme, schemes::LIGHT_SCHEME};

    #[test]
    fn test_default_schemes() {
        assert_eq!(
            LIGHT_SCHEME,
            DeviceScheme::new(
                DeviceKind::Light,
                &["/on", "/off"],
                &[
                    Hazard::FireHazard,
                    Hazard::ElectricEnergyConsumption,
                    Hazard::LogEnergyConsumption,
                    Hazard::LogUsageTime,
                    Hazard::PowerOutage,
                    Hazard::PowerSurge,
                ]
            )
        );
    }

    #[test]
    fn test_base_custom_scheme() {
        assert_eq!(
            DeviceScheme::base_custom_scheme("Thermostat"),
            DeviceScheme::new(DeviceKind::Custom(Cow::Borrowed("Thermostat")), &[], &[])
        );
    }

    #[test]
    fn test_custom_scheme_with_mandatory_routes() {
        assert_eq!(
            DeviceScheme::custom_scheme_with_mandatory_routes("Thermostat", &["/on", "/off"]),
            DeviceScheme::new(
                DeviceKind::Custom(Cow::Borrowed("Thermostat")),
                &["/on", "/off"],
                &[]
            )
        );
    }

    #[test]
    fn test_custom_scheme_with_allowed_hazards() {
        assert_eq!(
            DeviceScheme::custom_scheme_with_allowed_hazards(
                "Thermostat",
                &[Hazard::ElectricEnergyConsumption]
            ),
            DeviceScheme::new(
                DeviceKind::Custom(Cow::Borrowed("Thermostat")),
                &[],
                &[Hazard::ElectricEnergyConsumption]
            )
        );
    }

    #[test]
    fn test_custom_scheme() {
        assert_eq!(
            DeviceScheme::custom_scheme(
                "Thermostat",
                &["/on", "/off"],
                &[Hazard::ElectricEnergyConsumption]
            ),
            DeviceScheme::new(
                DeviceKind::Custom(Cow::Borrowed("Thermostat")),
                &["/on", "/off"],
                &[Hazard::ElectricEnergyConsumption]
            )
        );
    }

    #[test]
    fn test_getter_methods() {
        assert_eq!(LIGHT_SCHEME.kind, DeviceKind::Light);
        assert_eq!(LIGHT_SCHEME.mandatory_routes(), &["/on", "/off"]);
        assert_eq!(
            LIGHT_SCHEME.allowed_hazards(),
            &[
                Hazard::FireHazard,
                Hazard::ElectricEnergyConsumption,
                Hazard::LogEnergyConsumption,
                Hazard::LogUsageTime,
                Hazard::PowerOutage,
                Hazard::PowerSurge,
            ]
        );
    }
}

#[cfg(test)]
#[cfg(feature = "deserialize")]
mod tests {
    use alloc::borrow::Cow;

    use crate::route::{Route, RouteConfigs};

    use crate::economy::{Cost, CostTimespan, Costs, Economy, Roi, Rois};
    use crate::energy::{
        CarbonFootprint, CarbonFootprints, Energy, EnergyClass, EnergyEfficiencies,
        EnergyEfficiency, WaterUseEfficiency,
    };

    use crate::{deserialize, serialize};

    use super::{
        DeviceDescription, DeviceEnvironment, DeviceKind, DeviceMetrics, schemes::LIGHT_SCHEME,
    };

    fn energy() -> Energy {
        let energy_efficiencies =
            EnergyEfficiencies::init(EnergyEfficiency::new(-50, EnergyClass::A))
                .insert(EnergyEfficiency::new(50, EnergyClass::B));

        let carbon_footprints = CarbonFootprints::init(CarbonFootprint::new(-50, EnergyClass::A))
            .insert(CarbonFootprint::new(50, EnergyClass::B));

        let water_use_efficiency = WaterUseEfficiency::init_with_gpp(2.5)
            .penman_monteith_equation(3.2)
            .wer(1.1);

        Energy::init_with_energy_efficiencies(energy_efficiencies)
            .carbon_footprints(carbon_footprints)
            .water_use_efficiency(water_use_efficiency)
    }

    fn economy() -> Economy {
        let costs = Costs::init(Cost::new(100, CostTimespan::Week))
            .insert(Cost::new(1000, CostTimespan::Month));

        let roi = Rois::init(Roi::new(10, EnergyClass::A)).insert(Roi::new(20, EnergyClass::B));

        Economy::init_with_costs(costs).roi(roi)
    }

    fn routes() -> RouteConfigs {
        RouteConfigs::init(Route::put("On", "/on").serialize_data())
            .insert(Route::put("Off", "/off").serialize_data())
    }

    #[test]
    fn test_device_kind() {
        for device_kind in &[
            DeviceKind::Light,
            DeviceKind::Custom(Cow::Borrowed("Thermostat")),
        ] {
            assert_eq!(
                deserialize::<DeviceKind>(serialize(device_kind)),
                *device_kind
            );
        }
    }

    #[test]
    fn test_device_environment() {
        for device_environment in &[DeviceEnvironment::Os, DeviceEnvironment::Embedded] {
            assert_eq!(
                deserialize::<DeviceEnvironment>(serialize(device_environment)),
                *device_environment
            );
        }
    }

    #[test]
    fn test_device_metrics() {
        let energy_economy_metrics = DeviceMetrics::with_energy(energy()).add_economy(economy());

        assert_eq!(
            deserialize::<DeviceMetrics>(serialize(&energy_economy_metrics)),
            energy_economy_metrics
        );

        let economy_energy_metrics = DeviceMetrics::with_economy(economy()).add_energy(energy());

        assert_eq!(
            deserialize::<DeviceMetrics>(serialize(&economy_energy_metrics)),
            economy_energy_metrics
        );
    }

    #[test]
    fn test_device_description() {
        let device_description = DeviceDescription::new(LIGHT_SCHEME, "/light", routes())
            .text_description("A light device.");

        assert_eq!(
            deserialize::<DeviceDescription>(serialize(&device_description)),
            device_description
        );
    }
}
