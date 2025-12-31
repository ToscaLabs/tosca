// FIXME: Remove once passing by value will be supported in serde.
#![allow(clippy::trivially_copy_pass_by_ref)]

use alloc::borrow::Cow;
use alloc::string::String;

use hashbrown::DefaultHashBuilder;

use indexmap::map::{IndexMap, IntoIter, Iter, Keys};

use serde::{Deserialize, Serialize};

use crate::macros::map;

fn is_u8_max(value: &u8) -> bool {
    *value == u8::MAX
}

fn is_u8_min(value: &u8) -> bool {
    *value == u8::MIN
}

#[cfg(feature = "deserialize")]
fn u8_max() -> u8 {
    u8::MAX
}

fn is_u16_max(value: &u16) -> bool {
    *value == u16::MAX
}

fn is_u16_min(value: &u16) -> bool {
    *value == u16::MIN
}

#[cfg(feature = "deserialize")]
fn u16_max() -> u16 {
    u16::MAX
}

fn is_u32_max(value: &u32) -> bool {
    *value == u32::MAX
}

fn is_u32_min(value: &u32) -> bool {
    *value == u32::MIN
}

#[cfg(feature = "deserialize")]
fn u32_max() -> u32 {
    u32::MAX
}

fn is_u64_max(value: &u64) -> bool {
    *value == u64::MAX
}

fn is_u64_min(value: &u64) -> bool {
    *value == u64::MIN
}

#[cfg(feature = "deserialize")]
fn u64_max() -> u64 {
    u64::MAX
}

fn is_f32_max(value: &f32) -> bool {
    (f32::MAX - (*value).abs()) == 0.0
}

fn is_f32_min(value: &f32) -> bool {
    ((*value).abs() - f32::MIN.abs()) == 0.0
}

#[cfg(feature = "deserialize")]
fn f32_min() -> f32 {
    f32::MIN
}

#[cfg(feature = "deserialize")]
fn f32_max() -> f32 {
    f32::MAX
}

fn is_f64_max(value: &f64) -> bool {
    (f64::MAX - (*value).abs()) == 0.0
}

fn is_f64_min(value: &f64) -> bool {
    ((*value).abs() - f64::MIN.abs()) == 0.0
}

#[cfg(feature = "deserialize")]
fn f64_min() -> f64 {
    f64::MIN
}

#[cfg(feature = "deserialize")]
fn f64_max() -> f64 {
    f64::MAX
}

/// All supported kinds of route parameters.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub enum ParameterKind {
    /// A [`bool`] value.
    Bool {
        /// The initial [`bool`] value, also used as the default
        /// when the parameter is missing.
        default: bool,
    },
    /// An [`u8`] value.
    U8 {
        /// The initial [`u8`] value, also used as the default
        /// when the parameter is missing.
        default: u8,
        /// The minimum allowed [`u8`] value.
        #[serde(skip_serializing_if = "is_u8_max")]
        #[serde(default)]
        min: u8,
        /// The maximum allowed [`u8`] value.
        #[serde(skip_serializing_if = "is_u8_min")]
        #[serde(default = "u8_max")]
        max: u8,
    },
    /// An [`u16`] value.
    U16 {
        /// The initial [`u16`] value, also used as the default
        /// when the parameter is missing.
        default: u16,
        /// The minimum allowed [`u16`] value.
        #[serde(skip_serializing_if = "is_u16_max")]
        #[serde(default)]
        min: u16,
        /// The maximum allowed [`u16`] value.
        #[serde(skip_serializing_if = "is_u16_min")]
        #[serde(default = "u16_max")]
        max: u16,
    },
    /// An [`u32`] value.
    U32 {
        /// The initial [`u32`] value, also used as the default
        /// when the parameter is missing.
        default: u32,
        /// The minimum allowed [`u32`] value.
        #[serde(skip_serializing_if = "is_u32_max")]
        #[serde(default)]
        min: u32,
        /// The maximum allowed [`u32`] value.
        #[serde(skip_serializing_if = "is_u32_min")]
        #[serde(default = "u32_max")]
        max: u32,
    },
    /// An [`u64`] value.
    U64 {
        /// The initial [`u64`] value, also used as the default
        /// when the parameter is missing.
        default: u64,
        /// The minimum allowed [`u64`] value.
        #[serde(skip_serializing_if = "is_u64_max")]
        #[serde(default)]
        min: u64,
        /// The maximum allowed [`u64`] value.
        #[serde(skip_serializing_if = "is_u64_min")]
        #[serde(default = "u64_max")]
        max: u64,
    },
    /// A [`f32`] value.
    F32 {
        /// The initial [`f32`] value, also used as the default
        /// when the parameter is missing.
        default: f32,
        /// The minimum allowed [`f32`] value.
        #[serde(skip_serializing_if = "is_f32_max")]
        #[serde(default = "f32_min")]
        min: f32,
        /// The maximum allowed [`f32`] value.
        #[serde(skip_serializing_if = "is_f32_min")]
        #[serde(default = "f32_max")]
        max: f32,
        /// The decimal step for the [`f32`] value.
        #[serde(skip_serializing_if = "is_f32_min")]
        #[serde(default)]
        step: f32,
    },
    /// A [`f64`] value.
    F64 {
        /// The initial [`f64`] value, also used as the default
        /// when the parameter is missing.
        default: f64,
        /// The minimum allowed [`f64`] value.
        #[serde(skip_serializing_if = "is_f64_max")]
        #[serde(default = "f64_min")]
        min: f64,
        /// The maximum allowed [`f64`] value.
        #[serde(skip_serializing_if = "is_f64_min")]
        #[serde(default = "f64_max")]
        max: f64,
        /// The decimal step for the [`f64`] value.
        #[serde(skip_serializing_if = "is_f64_min")]
        #[serde(default)]
        step: f64,
    },
    /// A range of [`u32`] values.
    RangeU32 {
        /// The minimum allowed [`u32`] value.
        min: u32,
        /// The maximum allowed [`u32`] value.
        max: u32,
        /// The [`u32`] step between consecutive allowed values in the range.
        step: u32,
        /// The default [`u32`] value for the range.
        default: u32,
    },
    /// A range of [`u64`] values.
    RangeU64 {
        /// The minimum allowed [`u64`] value.
        min: u64,
        /// The maximum allowed [`u64`] value.
        max: u64,
        /// The [`u64`] step between consecutive allowed values in the range.
        step: u64,
        /// The default [`u64`] value for the range.
        default: u64,
    },
    /// A range of [`f64`] values.
    RangeF64 {
        /// The minimum allowed [`f64`] value.
        min: f64,
        /// The maximum allowed [`f64`] value.
        max: f64,
        /// The [`f64`] step between consecutive allowed values in the range.
        /// Always a positive value.
        step: f64,
        /// The default [`f64`] value for the range.
        default: f64,
    },
    /// A sequence of characters.
    CharsSequence {
        /// A sequence of characters representing the default value.
        default: Cow<'static, str>,
    },
}

impl ParameterKind {
    /// Returns the name of the [`ParameterKind`].
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Bool { .. } => "Bool",
            Self::U8 { .. } => "U8",
            Self::U16 { .. } => "U16",
            Self::U32 { .. } => "U32",
            Self::RangeU32 { .. } => "RangeU32",
            Self::U64 { .. } => "U64",
            Self::RangeU64 { .. } => "RangeU64",
            Self::F32 { .. } => "F32",
            Self::F64 { .. } => "F64",
            Self::RangeF64 { .. } => "RangeF64",
            Self::CharsSequence { .. } => "CharsSequence",
        }
    }

    /// Returns the type of the [`ParameterKind`].
    #[must_use]
    pub const fn as_type(&self) -> &'static str {
        match self {
            Self::Bool { .. } => "bool",
            Self::U8 { .. } => "u8",
            Self::U16 { .. } => "u16",
            Self::U32 { .. } | Self::RangeU32 { .. } => "u32",
            Self::U64 { .. } | Self::RangeU64 { .. } => "u64",
            Self::F32 { .. } => "f32",
            Self::F64 { .. } | Self::RangeF64 { .. } => "f64",
            Self::CharsSequence { .. } => "String",
        }
    }
}

/// The decimal precision of a floating-point value.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub enum DecimalPrecision {
    /// One digit.
    OneDigit,
    /// Two digits.
    TwoDigits,
    /// Three digits.
    ThreeDigits,
    /// Any sequence of digits.
    Any,
}

impl DecimalPrecision {
    const fn to_f32(self) -> f32 {
        match self {
            Self::OneDigit => 0.1,
            Self::TwoDigits => 0.01,
            Self::ThreeDigits => 0.001,
            Self::Any => 0.,
        }
    }

    const fn to_f64(self) -> f64 {
        match self {
            Self::OneDigit => 0.1,
            Self::TwoDigits => 0.01,
            Self::ThreeDigits => 0.001,
            Self::Any => 0.,
        }
    }
}

map! {
  /// A map associating each parameter name with its
  /// corresponding [`ParameterKind`].
  #[derive(Debug, Clone, PartialEq, Serialize)]
  #[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
  pub struct ParametersData(IndexMap<String, ParameterKind, DefaultHashBuilder>);
}

impl ParametersData {
    /// Retrieves a reference to the [`PatameterKind`] associated with the
    /// given key.
    #[must_use]
    #[inline]
    pub fn get(&self, key: &str) -> Option<&ParameterKind> {
        self.0.get(key)
    }
}

/// Route parameters.
#[derive(Debug, Clone)]
pub struct Parameters(IndexMap<&'static str, ParameterKind, DefaultHashBuilder>);

impl Default for Parameters {
    fn default() -> Self {
        Self::new()
    }
}

impl Parameters {
    /// Creates [`Parameters`].
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self(IndexMap::with_hasher(DefaultHashBuilder::default()))
    }

    /// Adds a [`bool`] parameter.
    #[must_use]
    #[inline]
    pub fn bool(self, name: &'static str, default: bool) -> Self {
        self.create_parameter(name, ParameterKind::Bool { default })
    }

    /// Adds an [`u8`] parameter.
    #[must_use]
    #[inline]
    pub fn u8(self, name: &'static str, default: u8) -> Self {
        self.create_parameter(
            name,
            ParameterKind::U8 {
                default,
                min: u8::MAX,
                max: u8::MIN,
            },
        )
    }

    /// Adds an [`u8`] parameter with specified limits.
    #[must_use]
    #[inline]
    pub fn u8_with_limits(self, name: &'static str, default: u8, min: u8, max: u8) -> Self {
        self.create_parameter(name, ParameterKind::U8 { default, min, max })
    }

    /// Adds an [`u16`] parameter.
    #[must_use]
    #[inline]
    pub fn u16(self, name: &'static str, default: u16) -> Self {
        self.create_parameter(
            name,
            ParameterKind::U16 {
                default,
                min: u16::MAX,
                max: u16::MIN,
            },
        )
    }

    /// Adds an [`u16`] parameter with specified limits.
    #[must_use]
    #[inline]
    pub fn u16_with_limits(self, name: &'static str, default: u16, min: u16, max: u16) -> Self {
        self.create_parameter(name, ParameterKind::U16 { default, min, max })
    }

    /// Adds an [`u32`] parameter.
    #[must_use]
    #[inline]
    pub fn u32(self, name: &'static str, default: u32) -> Self {
        self.create_parameter(
            name,
            ParameterKind::U32 {
                default,
                min: u32::MAX,
                max: u32::MIN,
            },
        )
    }

    /// Adds an [`u32`] parameter with specified limits.
    #[must_use]
    #[inline]
    pub fn u32_with_limits(self, name: &'static str, default: u32, min: u32, max: u32) -> Self {
        self.create_parameter(name, ParameterKind::U32 { default, min, max })
    }

    /// Adds an [`u64`] parameter.
    #[must_use]
    #[inline]
    pub fn u64(self, name: &'static str, default: u64) -> Self {
        self.create_parameter(
            name,
            ParameterKind::U64 {
                default,
                min: u64::MAX,
                max: u64::MIN,
            },
        )
    }

    /// Adds an [`u64`] parameter with specified limits.
    #[must_use]
    #[inline]
    pub fn u64_with_limits(self, name: &'static str, default: u64, min: u64, max: u64) -> Self {
        self.create_parameter(name, ParameterKind::U64 { default, min, max })
    }

    /// Adds a [`f32`] parameter.
    #[must_use]
    #[inline]
    pub fn f32(self, name: &'static str, default: f32) -> Self {
        self.create_parameter(
            name,
            ParameterKind::F32 {
                default,
                min: f32::MAX,
                max: f32::MIN,
                step: 0.,
            },
        )
    }

    /// Adds a [`f32`] parameter with specified limits.
    #[must_use]
    #[inline]
    pub fn f32_with_limits(
        self,
        name: &'static str,
        default: f32,
        min: f32,
        max: f32,
        decimal_precision: DecimalPrecision,
    ) -> Self {
        self.create_parameter(
            name,
            ParameterKind::F32 {
                default,
                min,
                max,
                step: decimal_precision.to_f32(),
            },
        )
    }

    /// Adds a [`f64`] parameter.
    #[must_use]
    #[inline]
    pub fn f64(self, name: &'static str, default: f64) -> Self {
        self.create_parameter(
            name,
            ParameterKind::F64 {
                default,
                min: f64::MAX,
                max: f64::MIN,
                step: 0.,
            },
        )
    }

    /// Adds a [`f64`] parameter with specified limits.
    #[must_use]
    #[inline]
    pub fn f64_with_limits(
        self,
        name: &'static str,
        default: f64,
        min: f64,
        max: f64,
        decimal_precision: DecimalPrecision,
    ) -> Self {
        self.create_parameter(
            name,
            ParameterKind::F64 {
                default,
                min,
                max,
                step: decimal_precision.to_f64(),
            },
        )
    }

    /// Adds an [`u32`] range without a default value.
    #[must_use]
    #[inline]
    pub fn rangeu32(self, name: &'static str, range: (u32, u32, u32)) -> Self {
        self.rangeu32_with_default(name, range, 0)
    }

    /// Adds an [`u32`] range with a default value.
    #[must_use]
    #[inline]
    pub fn rangeu32_with_default(
        self,
        name: &'static str,
        range: (u32, u32, u32),
        default: u32,
    ) -> Self {
        self.create_parameter(
            name,
            ParameterKind::RangeU32 {
                min: range.0,
                max: range.1,
                step: range.2,
                default,
            },
        )
    }

    /// Adds an [`u64`] range without a default value.
    #[must_use]
    #[inline]
    pub fn rangeu64(self, name: &'static str, range: (u64, u64, u64)) -> Self {
        self.rangeu64_with_default(name, range, 0)
    }

    /// Adds an [`u64`] range with a default value.
    #[must_use]
    #[inline]
    pub fn rangeu64_with_default(
        self,
        name: &'static str,
        range: (u64, u64, u64),
        default: u64,
    ) -> Self {
        self.create_parameter(
            name,
            ParameterKind::RangeU64 {
                min: range.0,
                max: range.1,
                step: range.2,
                default,
            },
        )
    }

    /// Adds a [`f64`] range without a default value.
    #[must_use]
    #[inline]
    pub fn rangef64(self, name: &'static str, range: (f64, f64, f64)) -> Self {
        self.rangef64_with_default(name, range, 0.0)
    }

    /// Adds a [`f64`] range with a default value.
    #[must_use]
    #[inline]
    pub fn rangef64_with_default(
        self,
        name: &'static str,
        range: (f64, f64, f64),
        default: f64,
    ) -> Self {
        self.create_parameter(
            name,
            ParameterKind::RangeF64 {
                min: range.0,
                max: range.1,
                step: range.2.abs(),
                default,
            },
        )
    }

    /// Adds a sequence of characters.
    #[must_use]
    #[inline]
    pub fn characters_sequence(
        self,
        name: &'static str,
        default: impl Into<Cow<'static, str>>,
    ) -> Self {
        self.create_parameter(
            name,
            ParameterKind::CharsSequence {
                default: default.into(),
            },
        )
    }

    /// Serializes [`Parameters`] data.
    ///
    /// **It consumes the parameter.**
    #[must_use]
    #[inline]
    pub fn serialize_data(self) -> ParametersData {
        let mut data = ParametersData::new();
        for (key, value) in self.0 {
            data.add(key.into(), value);
        }
        data
    }

    /// Checks if [`Parameters`] is empty.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterates over the names of [`Parameters`].
    #[must_use]
    #[inline]
    pub fn names(&self) -> Keys<'_, &str, ParameterKind> {
        self.0.keys()
    }

    fn create_parameter(mut self, name: &'static str, parameter_kind: ParameterKind) -> Self {
        self.0.insert(name, parameter_kind);
        self
    }
}

/// All supported parameter values extracted from the payload of an incoming
/// request linked to a route.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum ParameterValue {
    /// A [`bool`] value.
    Bool(bool),
    /// An [`u8`] value.
    U8(u8),
    /// An [`u16`] value.
    U16(u16),
    /// An [`u32`] value.
    U32(u32),
    /// An [`u64`] value.
    U64(u64),
    /// A [`f32`] value.
    F32(f32),
    /// A [`f64`] value.
    F64(f64),
    /// A sequence of characters.
    CharsSequence(Cow<'static, str>),
}

impl core::fmt::Display for ParameterValue {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Bool(v) => v.fmt(f),
            Self::U8(v) => v.fmt(f),
            Self::U16(v) => v.fmt(f),
            Self::U32(v) => v.fmt(f),
            Self::U64(v) => v.fmt(f),
            Self::F32(v) => v.fmt(f),
            Self::F64(v) => v.fmt(f),
            Self::CharsSequence(v) => v.fmt(f),
        }
    }
}

impl ParameterValue {
    /// Creates a [`ParameterValue`] from a [`ParameterKind`].
    #[must_use]
    pub fn from_parameter_kind(parameter_kind: &ParameterKind) -> Self {
        match parameter_kind {
            ParameterKind::Bool { default } => Self::Bool(*default),
            ParameterKind::U8 { default, .. } => Self::U8(*default),
            ParameterKind::U16 { default, .. } => Self::U16(*default),
            ParameterKind::U32 { default, .. } | ParameterKind::RangeU32 { default, .. } => {
                Self::U32(*default)
            }
            ParameterKind::U64 { default, .. } | ParameterKind::RangeU64 { default, .. } => {
                Self::U64(*default)
            }
            ParameterKind::F32 { default, .. } => Self::F32(*default),
            ParameterKind::F64 { default, .. } | ParameterKind::RangeF64 { default, .. } => {
                Self::F64(*default)
            }
            ParameterKind::CharsSequence { default, .. } => Self::CharsSequence(default.clone()),
        }
    }

    /// Returns the type of the [`ParameterValue`].
    #[must_use]
    pub const fn as_type(&self) -> &'static str {
        match self {
            Self::Bool(_) => "bool",
            Self::U8(_) => "u8",
            Self::U16(_) => "u16",
            Self::U32(_) => "u32",
            Self::U64(_) => "u64",
            Self::F32(_) => "f32",
            Self::F64(_) => "f64",
            Self::CharsSequence(_) => "String",
        }
    }

    /// Checks if the [`ParameterValue`] matches the given [`ParameterKind`].
    #[must_use]
    pub const fn match_kind(&self, parameter_kind: &ParameterKind) -> bool {
        matches!(
            (self, parameter_kind),
            (Self::Bool(_), ParameterKind::Bool { .. })
                | (Self::U8(_), ParameterKind::U8 { .. })
                | (Self::U16(_), ParameterKind::U16 { .. })
                | (
                    Self::U32(_),
                    ParameterKind::U32 { .. } | ParameterKind::RangeU32 { .. }
                )
                | (
                    Self::U64(_),
                    ParameterKind::U64 { .. } | ParameterKind::RangeU64 { .. }
                )
                | (Self::F32(_), ParameterKind::F32 { .. })
                | (
                    Self::F64(_),
                    ParameterKind::F64 { .. } | ParameterKind::RangeF64 { .. }
                )
                | (Self::CharsSequence(_), ParameterKind::CharsSequence { .. })
        )
    }
}

/// A map associating each parameter name with its
/// corresponding [`ParameterValue`].
#[derive(Debug, PartialEq, Deserialize)]
pub struct ParametersValues<'a>(IndexMap<Cow<'a, str>, ParameterValue, DefaultHashBuilder>);

impl Default for ParametersValues<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> IntoIterator for ParametersValues<'a> {
    type Item = (Cow<'a, str>, ParameterValue);
    type IntoIter = IntoIter<Cow<'a, str>, ParameterValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a ParametersValues<'a> {
    type Item = (&'a Cow<'a, str>, &'a ParameterValue);
    type IntoIter = Iter<'a, Cow<'a, str>, ParameterValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> ParametersValues<'a> {
    /// Creates [`ParametersValues`].
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self(IndexMap::with_hasher(DefaultHashBuilder::default()))
    }

    /// Adds a [`ParameterValue`].
    #[inline]
    pub fn parameter_value(
        &mut self,
        name: impl Into<Cow<'a, str>>,
        parameter_value: ParameterValue,
    ) -> &mut Self {
        self.0.insert(name.into(), parameter_value);
        self
    }

    /// Adds a [`bool`] value.
    #[inline]
    pub fn bool(&mut self, name: impl Into<Cow<'a, str>>, value: bool) -> &mut Self {
        self.parameter_value(name, ParameterValue::Bool(value))
    }

    /// Adds an [`u8`] parameter.
    #[inline]
    pub fn u8(&mut self, name: impl Into<Cow<'a, str>>, value: u8) -> &mut Self {
        self.parameter_value(name, ParameterValue::U8(value))
    }

    /// Adds an [`u16`] parameter.
    #[inline]
    pub fn u16(&mut self, name: impl Into<Cow<'a, str>>, value: u16) -> &mut Self {
        self.parameter_value(name, ParameterValue::U16(value))
    }

    /// Adds an [`u32`] parameter.
    #[inline]
    pub fn u32(&mut self, name: impl Into<Cow<'a, str>>, value: u32) -> &mut Self {
        self.parameter_value(name, ParameterValue::U32(value))
    }

    /// Adds an [`u64`] parameter.
    #[inline]
    pub fn u64(&mut self, name: impl Into<Cow<'a, str>>, value: u64) -> &mut Self {
        self.parameter_value(name, ParameterValue::U64(value))
    }

    /// Adds a [`f32`] parameter.
    #[inline]
    pub fn f32(&mut self, name: impl Into<Cow<'a, str>>, value: f32) -> &mut Self {
        self.parameter_value(name, ParameterValue::F32(value))
    }

    /// Adds a [`f64`] parameter.
    #[inline]
    pub fn f64(&mut self, name: impl Into<Cow<'a, str>>, value: f64) -> &mut Self {
        self.parameter_value(name, ParameterValue::F64(value))
    }

    /// Adds a sequence of characters.
    #[inline]
    pub fn characters_sequence(
        &mut self,
        name: impl Into<Cow<'a, str>>,
        value: String,
    ) -> &mut Self {
        self.parameter_value(name, ParameterValue::CharsSequence(value.into()))
    }

    /// Retrieves a [`ParameterValue`] by name.
    ///
    /// Returns [`None`] if the parameter does not exist.
    #[must_use]
    #[inline]
    pub fn get<'b>(&'b self, name: impl Into<Cow<'b, str>>) -> Option<&'b ParameterValue> {
        self.0.get(&name.into())
    }

    /// Returns an iterator over [`ParameterValue`]s.
    ///
    /// **Iterates over the elements in the order they were inserted.**
    #[must_use]
    #[inline]
    pub fn iter(&self) -> Iter<'_, Cow<'_, str>, ParameterValue> {
        self.0.iter()
    }
}

/// Parameter data payload, consisting of a [`ParameterKind`] and a
/// [`ParameterValue`].
pub struct ParameterPayload {
    /// Parameter kind.
    pub kind: ParameterKind,
    /// Parameter value.
    pub value: ParameterValue,
}

impl ParameterPayload {
    /// Creates a [`ParameterPayload`].
    #[must_use]
    pub const fn new(kind: ParameterKind, value: ParameterValue) -> Self {
        Self { kind, value }
    }
}

map! {
  /// A map associating each parameter name with its
  /// corresponding [`ParameterPayload`].
  pub struct ParametersPayloads<'a>(IndexMap<Cow<'a, str>, ParameterPayload, DefaultHashBuilder>);
}

impl<'a> ParametersPayloads<'a> {
    /// Retrieves a [`ParameterPayload`] by name.
    ///
    /// Returns [`None`] if the parameter does not exist.
    #[must_use]
    #[inline]
    pub fn get<'b>(&'b self, name: impl Into<Cow<'b, str>>) -> Option<&'b ParameterPayload> {
        self.0.get(&name.into())
    }

    /// Extracts a [`ParameterPayload`] by name.
    ///
    /// **It consumes the parameter.**
    ///
    /// Returns [`None`] if the parameter does not exist.
    #[must_use]
    #[inline]
    pub fn extract(&mut self, name: impl Into<Cow<'a, str>>) -> Option<ParameterPayload> {
        self.0.swap_remove(&name.into())
    }
}

#[cfg(test)]
#[cfg(feature = "deserialize")]
mod tests {
    use alloc::string::String;

    use crate::{deserialize, serialize};

    use super::{ParameterKind, Parameters, ParametersData, ParametersValues};

    fn expected_parameters_data() -> ParametersData {
        ParametersData::new()
            .insert("bool".into(), ParameterKind::Bool { default: true })
            .insert(
                "u8".into(),
                ParameterKind::U8 {
                    default: 0,
                    min: u8::MIN,
                    max: u8::MAX,
                },
            )
            .insert(
                "u16".into(),
                ParameterKind::U16 {
                    default: 0,
                    min: u16::MIN,
                    max: u16::MAX,
                },
            )
            .insert(
                "u32".into(),
                ParameterKind::U32 {
                    default: 0,
                    min: u32::MIN,
                    max: u32::MAX,
                },
            )
            .insert(
                "u64".into(),
                ParameterKind::U64 {
                    default: 0,
                    min: u64::MIN,
                    max: u64::MAX,
                },
            )
            .insert(
                "f32".into(),
                ParameterKind::F32 {
                    default: 0.,
                    min: f32::MIN,
                    max: f32::MAX,
                    step: 0.,
                },
            )
            .insert(
                "f64".into(),
                ParameterKind::F64 {
                    default: 0.,
                    min: f64::MIN,
                    max: f64::MAX,
                    step: 0.,
                },
            )
            .insert(
                "rangeu32".into(),
                ParameterKind::RangeU32 {
                    min: 0,
                    max: 20,
                    step: 1,
                    default: 5,
                },
            )
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
                    default: 5.,
                },
            )
            .insert(
                "greeting".into(),
                ParameterKind::CharsSequence {
                    default: "hello".into(),
                },
            )
            .insert(
                "greeting2".into(),
                ParameterKind::CharsSequence {
                    default: "hello".into(),
                },
            )
    }

    #[test]
    fn test_parameters() {
        let parameters = Parameters::new()
            .bool("bool", true)
            .u8("u8", 0)
            .u16("u16", 0)
            .u32("u32", 0)
            .u64("u64", 0)
            .f32("f32", 0.)
            .f64("f64", 0.)
            .rangeu32_with_default("rangeu32", (0, 20, 1), 5)
            .rangeu64_with_default("rangeu64", (0, 20, 1), 5)
            .rangef64_with_default("rangef64", (0., 20., 0.1), 5.)
            .characters_sequence("greeting", "hello")
            .characters_sequence("greeting2", String::from("hello"))
            // Adds a duplicate to see whether that value is maintained or
            // removed.
            .u16("u16", 0);

        assert_eq!(
            deserialize::<ParametersData>(serialize(parameters.serialize_data())),
            expected_parameters_data(),
        );
    }

    #[test]
    fn test_deserialize_parameters_values() {
        let mut parameters = ParametersValues::new();
        parameters.bool("one", true);
        parameters.u8("two", 8);
        parameters.f32("three", 3.0);

        let json_value = serde_json::json!({
            "one": true,
            "two": 8,
            "three": 3.0,
        });

        assert_eq!(deserialize::<ParametersValues>(json_value), parameters);
    }
}
