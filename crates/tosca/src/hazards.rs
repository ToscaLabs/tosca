use hashbrown::DefaultHashBuilder;

use indexmap::set::{IndexSet, IntoIter, Iter};

use serde::{Deserialize, Serialize};

use crate::macros::set;

/// All [`Hazard`]s.
pub const ALL_HAZARDS: &[Hazard] = &[
    Hazard::AirPoisoning,
    Hazard::Asphyxia,
    Hazard::AudioVideoDisplay,
    Hazard::AudioVideoRecordAndStore,
    Hazard::ElectricEnergyConsumption,
    Hazard::Explosion,
    Hazard::FireHazard,
    Hazard::GasConsumption,
    Hazard::LogEnergyConsumption,
    Hazard::LogUsageTime,
    Hazard::PaySubscriptionFee,
    Hazard::PowerOutage,
    Hazard::PowerSurge,
    Hazard::RecordIssuedCommands,
    Hazard::RecordUserPreferences,
    Hazard::SpendMoney,
    Hazard::SpoiledFood,
    Hazard::TakeDeviceScreenshots,
    Hazard::TakePictures,
    Hazard::UnauthorisedPhysicalAccess,
    Hazard::VideoDisplay,
    Hazard::VideoRecordAndStore,
    Hazard::WaterConsumption,
    Hazard::WaterFlooding,
];

/// All possible hazards for a device route.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub enum Hazard {
    /// The execution may release toxic gases.
    AirPoisoning,
    /// The execution may cause oxygen deficiency by gaseous substances.
    Asphyxia,
    /// The execution authorises an application to display a video
    /// with audio coming from a device.
    AudioVideoDisplay,
    /// The execution authorises an application to record and save a video with
    /// audio coming from a device on persistent storage.
    AudioVideoRecordAndStore,
    /// The execution enables a device which consumes electricity.
    ElectricEnergyConsumption,
    /// The execution may cause an explosion.
    Explosion,
    /// The execution may cause fire.
    FireHazard,
    /// The execution enables a device which consumes gas.
    GasConsumption,
    /// The execution authorises an application to get and save information
    /// about a device's energy impact.
    LogEnergyConsumption,
    /// The execution authorises an application to get and save information
    /// about a device's duration of use.
    LogUsageTime,
    /// The execution authorises an application to use payment information and
    /// make a periodic payment.
    PaySubscriptionFee,
    /// The execution may cause an interruption in the supply of electricity.
    PowerOutage,
    /// The execution may lead to exposure to high voltages.
    PowerSurge,
    /// The execution authorises an application to get and save user inputs.
    RecordIssuedCommands,
    /// The execution authorises an application to get and save information
    /// about user's preferences.
    RecordUserPreferences,
    /// The execution authorises an application to use payment information and
    /// make a payment transaction.
    SpendMoney,
    /// The execution may lead to rotten food.
    SpoiledFood,
    /// The execution authorises an application to read and take screenshots
    /// from the display output.
    TakeDeviceScreenshots,
    /// The execution authorises an application to use a camera and take photos.
    TakePictures,
    /// The execution disables a protection mechanism, therefore unauthorised
    /// individuals may physically access to the environment.
    UnauthorisedPhysicalAccess,
    /// The execution authorises an application to display a video coming from
    /// a device.
    VideoDisplay,
    /// The execution authorises an application to record and save a video
    /// coming from a device on persistent storage.
    VideoRecordAndStore,
    /// The execution enables a device which consumes water.
    WaterConsumption,
    /// The execution enables a device to water usage, which may lead to flood.
    WaterFlooding,
}

impl core::convert::AsRef<Self> for Hazard {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl core::fmt::Debug for Hazard {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.name().fmt(f)
    }
}

impl core::fmt::Display for Hazard {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.name().fmt(f)
    }
}

impl Hazard {
    /// Returns the [`Hazard`] name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::AirPoisoning => "Air Poisoning",
            Self::Asphyxia => "Asphyxia",
            Self::AudioVideoDisplay => "Audio Video Display",
            Self::AudioVideoRecordAndStore => "Audio Video Record And Store",
            Self::ElectricEnergyConsumption => "Electric Energy Consumption",
            Self::Explosion => "Explosion",
            Self::FireHazard => "Fire Hazard",
            Self::GasConsumption => "Gas Consumption",
            Self::LogEnergyConsumption => "Log Energy Consumption",
            Self::LogUsageTime => "Log Usage Time",
            Self::PaySubscriptionFee => "Pay Subscription Fee",
            Self::PowerOutage => "Power Outage",
            Self::PowerSurge => "Power Surge",
            Self::RecordIssuedCommands => "Record Issued Commands",
            Self::RecordUserPreferences => "Record User Preferences",
            Self::SpendMoney => "Spend Money",
            Self::SpoiledFood => "Spoiled Food",
            Self::TakeDeviceScreenshots => "Take Device Screenshots",
            Self::TakePictures => "Take Pictures",
            Self::UnauthorisedPhysicalAccess => "Unauthorised Physical Access",
            Self::VideoDisplay => "Video Display",
            Self::VideoRecordAndStore => "Video Record and Store",
            Self::WaterConsumption => "Water Consumption",
            Self::WaterFlooding => "Water Flooding",
        }
    }

    /// Returns the [`Hazard`] description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::AirPoisoning => "The execution may release toxic gases.",
            Self::Asphyxia => "The execution may cause oxygen deficiency by gaseous substances.",
            Self::AudioVideoDisplay => {
                "The execution authorises an application to display a video with audio coming from a device."
            }
            Self::AudioVideoRecordAndStore => {
                "The execution authorises an application to record and save a video with audio coming from a device on persistent storage."
            }
            Self::ElectricEnergyConsumption => {
                "The execution enables a device which consumes electricity."
            }
            Self::Explosion => "The execution may cause an explosion.",
            Self::FireHazard => "The execution may cause fire.",
            Self::GasConsumption => "The execution enables a device which consumes gas.",
            Self::LogEnergyConsumption => {
                "The execution authorises an application to get and save information about a device's energy impact."
            }
            Self::LogUsageTime => {
                "The execution authorises an application to get and save information about a device's duration of use."
            }
            Self::PaySubscriptionFee => {
                "The execution authorises an application to use payment information and make a periodic payment."
            }
            Self::PowerOutage => {
                "The execution may cause an interruption in the supply of electricity."
            }
            Self::PowerSurge => "The execution may lead to exposure to high voltages.",
            Self::RecordIssuedCommands => {
                "The execution authorises an application to get and save user inputs."
            }
            Self::RecordUserPreferences => {
                "The execution authorises an application to get and save information about user's preferences."
            }
            Self::SpendMoney => {
                "The execution authorises an application to use payment information and make a payment transaction."
            }
            Self::SpoiledFood => "The execution may lead to rotten food.",
            Self::TakeDeviceScreenshots => {
                "The execution authorises an application to read and take screenshots from the display output."
            }
            Self::TakePictures => {
                "The execution authorises an application to use a camera and take photos."
            }
            Self::UnauthorisedPhysicalAccess => {
                "The execution disables a protection mechanism, therefore unauthorised individuals may physically access to the environment."
            }
            Self::VideoDisplay => {
                "The execution authorises an application to display a video coming from a device."
            }
            Self::VideoRecordAndStore => {
                "The execution authorises an application to record and save a video coming from a device on persistent storage."
            }
            Self::WaterConsumption => "The execution enables a device which consumes water.",
            Self::WaterFlooding => {
                "The execution enables a device to water usage, which may lead to flood."
            }
        }
    }

    /// Returns the [`Category`] associated with the [`Hazard`].
    ///
    /// A hazard is **always** associated with **exactly one** one category.
    #[must_use]
    pub const fn category(&self) -> Category {
        match self {
            Self::AirPoisoning
            | Self::Asphyxia
            | Self::Explosion
            | Self::FireHazard
            | Self::PowerOutage
            | Self::PowerSurge
            | Self::SpoiledFood
            | Self::UnauthorisedPhysicalAccess
            | Self::WaterFlooding => Category::Safety,
            Self::AudioVideoDisplay
            | Self::AudioVideoRecordAndStore
            | Self::LogEnergyConsumption
            | Self::LogUsageTime
            | Self::RecordIssuedCommands
            | Self::RecordUserPreferences
            | Self::TakeDeviceScreenshots
            | Self::TakePictures
            | Self::VideoDisplay
            | Self::VideoRecordAndStore => Category::Privacy,
            Self::ElectricEnergyConsumption
            | Self::GasConsumption
            | Self::PaySubscriptionFee
            | Self::SpendMoney
            | Self::WaterConsumption => Category::Financial,
        }
    }

    /// Returns the identifier associated with the [`Hazard`].
    #[must_use]
    pub const fn id(&self) -> u16 {
        match self {
            Self::AirPoisoning => 0,
            Self::Asphyxia => 1,
            Self::AudioVideoDisplay => 2,
            Self::AudioVideoRecordAndStore => 3,
            Self::ElectricEnergyConsumption => 4,
            Self::Explosion => 5,
            Self::FireHazard => 6,
            Self::GasConsumption => 7,
            Self::LogEnergyConsumption => 8,
            Self::LogUsageTime => 9,
            Self::PaySubscriptionFee => 10,
            Self::PowerOutage => 11,
            Self::PowerSurge => 12,
            Self::RecordIssuedCommands => 13,
            Self::RecordUserPreferences => 14,
            Self::SpendMoney => 15,
            Self::SpoiledFood => 16,
            Self::TakeDeviceScreenshots => 17,
            Self::TakePictures => 18,
            Self::UnauthorisedPhysicalAccess => 19,
            Self::VideoDisplay => 20,
            Self::VideoRecordAndStore => 21,
            Self::WaterConsumption => 22,
            Self::WaterFlooding => 23,
        }
    }

    /// Returns the [`Hazard`] associated with the given integer identifier.
    ///
    /// The return value is [`None`] when the identifier is invalid or does
    /// not exist.
    #[must_use]
    pub const fn from_id(id: u16) -> Option<Self> {
        match id {
            0 => Some(Self::AirPoisoning),
            1 => Some(Self::Asphyxia),
            2 => Some(Self::AudioVideoDisplay),
            3 => Some(Self::AudioVideoRecordAndStore),
            4 => Some(Self::ElectricEnergyConsumption),
            5 => Some(Self::Explosion),
            6 => Some(Self::FireHazard),
            7 => Some(Self::GasConsumption),
            8 => Some(Self::LogEnergyConsumption),
            9 => Some(Self::LogUsageTime),
            10 => Some(Self::PaySubscriptionFee),
            11 => Some(Self::PowerOutage),
            12 => Some(Self::PowerSurge),
            13 => Some(Self::RecordIssuedCommands),
            14 => Some(Self::RecordUserPreferences),
            15 => Some(Self::SpendMoney),
            16 => Some(Self::SpoiledFood),
            17 => Some(Self::TakeDeviceScreenshots),
            18 => Some(Self::TakePictures),
            19 => Some(Self::UnauthorisedPhysicalAccess),
            20 => Some(Self::VideoDisplay),
            21 => Some(Self::VideoRecordAndStore),
            22 => Some(Self::WaterConsumption),
            23 => Some(Self::WaterFlooding),
            _ => None,
        }
    }

    /// Returns the [`HazardData`] constructed from the given [`Hazard`].
    #[must_use]
    pub const fn data(&self) -> HazardData {
        HazardData {
            id: self.id(),
            name: self.name(),
            description: self.description(),
            category_name: self.category().name(),
            category_description: self.category().description(),
        }
    }
}

set! {
  /// A collection of [`Hazard`]s.
  #[derive(Debug, Clone, PartialEq, Serialize)]
  #[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
  pub struct Hazards(IndexSet<Hazard, DefaultHashBuilder>);
}

impl Hazards {
    /// Constructs [`Hazards`] from an array of [`Hazard`]s.
    #[must_use]
    #[inline]
    pub fn init_from_hazards<const N: usize>(input_elements: [Hazard; N]) -> Self {
        let mut elements = Self::new();
        for element in input_elements {
            elements.add(element);
        }
        elements
    }
}

/// All [`Hazard`] data.
#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
pub struct HazardData {
    /// Identifier.
    pub id: u16,
    /// Name.
    pub name: &'static str,
    /// Description.
    pub description: &'static str,
    /// Category name.
    pub category_name: &'static str,
    /// Category description.
    pub category_description: &'static str,
}

/// All [`Category`]s.
pub const ALL_CATEGORIES: &[Category] = &[Category::Safety, Category::Privacy, Category::Financial];

/// Hazard categories.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    /// Category including all financial-related hazards.
    Financial,
    /// Category including all privacy-related hazards.
    Privacy,
    /// Category including all safety-related hazards.
    Safety,
}

impl core::fmt::Debug for Category {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.name().fmt(f)
    }
}

impl core::fmt::Display for Category {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.name().fmt(f)
    }
}

impl Category {
    /// Returns a [`Category`] name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Financial => "Financial",
            Self::Privacy => "Privacy",
            Self::Safety => "Safety",
        }
    }

    /// Returns a [`Category`] description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Financial => "Category which includes all financial-related hazards.",
            Self::Privacy => "Category which includes all privacy-related hazards.",
            Self::Safety => "Category which includes all safety-related hazards.",
        }
    }

    /// Returns all [`Hazard`]s associated with a [`Category`].
    #[must_use]
    pub const fn hazards(&self) -> &[Hazard] {
        match self {
            Self::Financial => &[
                Hazard::ElectricEnergyConsumption,
                Hazard::GasConsumption,
                Hazard::PaySubscriptionFee,
                Hazard::SpendMoney,
                Hazard::WaterConsumption,
            ],
            Self::Privacy => &[
                Hazard::AudioVideoDisplay,
                Hazard::AudioVideoRecordAndStore,
                Hazard::LogEnergyConsumption,
                Hazard::LogUsageTime,
                Hazard::RecordIssuedCommands,
                Hazard::RecordUserPreferences,
                Hazard::TakeDeviceScreenshots,
                Hazard::TakePictures,
                Hazard::VideoDisplay,
                Hazard::VideoRecordAndStore,
            ],
            Self::Safety => &[
                Hazard::AirPoisoning,
                Hazard::Asphyxia,
                Hazard::Explosion,
                Hazard::FireHazard,
                Hazard::PowerOutage,
                Hazard::PowerSurge,
                Hazard::SpoiledFood,
                Hazard::UnauthorisedPhysicalAccess,
                Hazard::WaterFlooding,
            ],
        }
    }
}

#[cfg(test)]
#[cfg(feature = "deserialize")]
mod tests {
    use crate::{deserialize, serialize};

    use super::{ALL_CATEGORIES, ALL_HAZARDS, Category, Hazard};

    #[test]
    fn test_hazard() {
        // Check wrong id. 1000 will be always a big value.
        assert_eq!(Hazard::from_id(1000), None);

        // Compare all hazards.
        for hazard in ALL_HAZARDS {
            assert_eq!(Hazard::from_id(hazard.id()), Some(*hazard));
            assert_eq!(
                serialize(hazard.data()),
                serde_json::json!({
                        "id": hazard.id(),
                        "name": hazard.name(),
                        "description": hazard.description(),
                        "category_name": hazard.category().name(),
                        "category_description": hazard.category().description(),
                    }


                )
            );
            assert_eq!(deserialize::<Hazard>(serialize(hazard)), *hazard);
        }
    }

    #[test]
    fn test_category() {
        // Compare all categories.
        for category in ALL_CATEGORIES {
            assert_eq!(deserialize::<Category>(serialize(category)), *category);
        }
    }
}
