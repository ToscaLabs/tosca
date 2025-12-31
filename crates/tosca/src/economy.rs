use hashbrown::DefaultHashBuilder;

use indexmap::set::{IndexSet, IntoIter, Iter};

use serde::Serialize;

use crate::energy::EnergyClass;
use crate::macros::set;

/// Timespan selected to estimate the device costs.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub enum CostTimespan {
    /// Week.
    Week,
    /// Month.
    Month,
    /// Year.
    Year,
}

impl CostTimespan {
    const fn name(self) -> &'static str {
        match self {
            Self::Week => "week",
            Self::Month => "month",
            Self::Year => "year",
        }
    }
}

impl core::fmt::Display for CostTimespan {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.name().fmt(f)
    }
}

/// Device cost.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub struct Cost {
    /// Amount of money in USD currency.
    /// A negative value indicates savings during the considered
    /// timespan, while a positive value indicates expenditures in the
    /// considered timespan.
    #[serde(rename = "usd")]
    pub usd_currency: i32,
    /// Considered timespan to estimate the costs.
    pub timespan: CostTimespan,
}

impl core::fmt::Display for Cost {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "The device {} {} USD in a {} timespan",
            if self.usd_currency < 0 {
                "saves"
            } else {
                "spends"
            },
            self.usd_currency.abs(),
            self.timespan
        )
    }
}

impl Cost {
    /// Creates a [`Cost`].
    #[must_use]
    pub const fn new(usd_currency: i32, timespan: CostTimespan) -> Self {
        Self {
            usd_currency,
            timespan,
        }
    }
}

set! {
  /// All device [`Cost`]s.
  #[derive(Debug, Clone, PartialEq, Serialize)]
  #[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
  pub struct Costs(IndexSet<Cost, DefaultHashBuilder>);
}

/// Return on investments (ROI).
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub struct Roi {
    /// Number of years used to calculate the ROI.
    pub years: u8,
    /// Energy class.
    #[serde(rename = "energy-class")]
    pub energy_class: EnergyClass,
}

impl core::fmt::Display for Roi {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "The device has a Return on Investments (Roi) for the `{}` \
            energy efficiency class over a timespan of {} {}",
            self.energy_class,
            self.years,
            if self.years > 1 { "years" } else { "year" },
        )
    }
}

impl Roi {
    /// Creates a [`Roi`].
    ///
    /// If the `years` parameter is set to **0**, it will automatically be
    /// adjusted to **1**
    /// If the `years` parameter exceeds **30**, it will automatically be
    /// adjusted to **30**.
    #[must_use]
    pub const fn new(years: u8, energy_class: EnergyClass) -> Self {
        let years = match years {
            0 => 1,
            30.. => 30,
            _ => years,
        };
        Self {
            years,
            energy_class,
        }
    }
}

set! {
  /// All device [`Roi`]s.
  #[derive(Debug, Clone, PartialEq, Serialize)]
  #[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
  pub struct Rois(IndexSet<Roi, DefaultHashBuilder>);
}

/// Economy data related to a device.
#[derive(Debug, PartialEq, Clone, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub struct Economy {
    /// Costs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub costs: Option<Costs>,
    /// Return on investments (ROI).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roi: Option<Rois>,
}

impl Economy {
    /// Creates an empty [`Economy`].
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            costs: None,
            roi: None,
        }
    }

    /// Creates an [`Economy`] initialized with the [`Costs`] data.
    #[must_use]
    pub const fn init_with_costs(costs: Costs) -> Self {
        Self {
            costs: Some(costs),
            roi: None,
        }
    }

    /// Creates an [`Economy`] initialized with the [`Rois`] data.
    #[must_use]
    pub const fn init_with_roi(roi: Rois) -> Self {
        Self {
            costs: None,
            roi: Some(roi),
        }
    }

    /// Adds the [`Costs`] data.
    #[must_use]
    #[inline]
    pub fn costs(mut self, costs: Costs) -> Self {
        self.costs = Some(costs);
        self
    }

    /// Adds the [`Rois`] data.
    #[must_use]
    #[inline]
    pub fn roi(mut self, roi: Rois) -> Self {
        self.roi = Some(roi);
        self
    }

    /// Checks if [`Economy`] is **entirely** empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.costs.is_none() && self.roi.is_none()
    }
}

#[cfg(test)]
#[cfg(feature = "deserialize")]
mod tests {
    use super::Economy;

    use crate::energy::EnergyClass;
    use crate::{deserialize, serialize};

    use super::{Cost, CostTimespan, Costs, Roi, Rois};

    #[test]
    fn test_cost_timespan() {
        for cost_timespan in &[CostTimespan::Week, CostTimespan::Month, CostTimespan::Year] {
            assert_eq!(
                deserialize::<CostTimespan>(serialize(cost_timespan)),
                *cost_timespan
            );
        }
    }

    #[test]
    fn test_cost() {
        let cost = Cost::new(100, CostTimespan::Week);

        assert_eq!(deserialize::<Cost>(serialize(cost)), cost);
    }

    #[test]
    fn test_roi_serde() {
        let roi = Roi::new(10, EnergyClass::A);

        assert_eq!(deserialize::<Roi>(serialize(roi)), roi);
    }

    #[test]
    fn test_roi_clamping() {
        assert_eq!(Roi::new(0, EnergyClass::A).years, 1);
        assert_eq!(Roi::new(31, EnergyClass::A).years, 30);
        assert_eq!(Roi::new(20, EnergyClass::A).years, 20);
    }

    #[test]
    fn test_economy() {
        let mut economy = Economy::empty();

        let costs = Costs::init(Cost::new(100, CostTimespan::Week))
            .insert(Cost::new(1000, CostTimespan::Month));

        let roi = Rois::init(Roi::new(10, EnergyClass::A)).insert(Roi::new(20, EnergyClass::B));

        assert!(economy.is_empty());

        economy = economy.costs(costs).roi(roi);

        assert_eq!(deserialize::<Economy>(serialize(&economy)), economy);
    }
}
