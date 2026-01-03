use alloc::borrow::Cow;
use alloc::format;

use tosca::parameters::{
    ParameterKind, ParameterPayload, ParameterValue, ParametersPayloads as ToscaParametersPayloads,
};

use crate::response::ErrorResponse;
use crate::server::invalid_data;

/// A [`bool`] payload.
pub struct BoolPayload {
    /// Value.
    pub value: bool,
    /// Default value.
    pub default: bool,
}

impl BoolPayload {
    const fn new(value: bool, default: bool) -> Self {
        Self { value, default }
    }
}

/// An [`u8`] payload.
pub struct U8Payload {
    /// Value.
    pub value: u8,
    /// Default value.
    pub default: u8,
    /// Minimum value.
    pub min: u8,
    /// Maximum value.
    pub max: u8,
}

impl U8Payload {
    const fn new(value: u8, default: u8, min: u8, max: u8) -> Self {
        Self {
            value,
            default,
            min,
            max,
        }
    }
}

/// An [`u16`] payload.
pub struct U16Payload {
    /// Value.
    pub value: u16,
    /// Default value.
    pub default: u16,
    /// Minimum value.
    pub min: u16,
    /// Maximum value.
    pub max: u16,
}

impl U16Payload {
    const fn new(value: u16, default: u16, min: u16, max: u16) -> Self {
        Self {
            value,
            default,
            min,
            max,
        }
    }
}

/// An [`u32`] payload.
pub struct U32Payload {
    /// Value.
    pub value: u32,
    /// Default value.
    pub default: u32,
    /// Minimum value.
    pub min: u32,
    /// Maximum value.
    pub max: u32,
}

impl U32Payload {
    const fn new(value: u32, default: u32, min: u32, max: u32) -> Self {
        Self {
            value,
            default,
            min,
            max,
        }
    }
}

/// An [`u64`] payload.
pub struct U64Payload {
    /// Value.
    pub value: u64,
    /// Default value.
    pub default: u64,
    /// Minimum value.
    pub min: u64,
    /// Maximum value.
    pub max: u64,
}

impl U64Payload {
    const fn new(value: u64, default: u64, min: u64, max: u64) -> Self {
        Self {
            value,
            default,
            min,
            max,
        }
    }
}

/// A [`f32`] payload.
pub struct F32Payload {
    /// Value.
    pub value: f32,
    /// Default value.
    pub default: f32,
    /// Minimum value.
    pub min: f32,
    /// Maximum value.
    pub max: f32,
    /// Step.
    pub step: f32,
}

impl F32Payload {
    const fn new(value: f32, default: f32, min: f32, max: f32, step: f32) -> Self {
        Self {
            value,
            default,
            min,
            max,
            step,
        }
    }
}

/// A [`f64`] payload.
pub struct F64Payload {
    /// Value.
    pub value: f64,
    /// Default value.
    pub default: f64,
    /// Minimum value.
    pub min: f64,
    /// Maximum value.
    pub max: f64,
    /// Step.
    pub step: f64,
}

impl F64Payload {
    const fn new(value: f64, default: f64, min: f64, max: f64, step: f64) -> Self {
        Self {
            value,
            default,
            min,
            max,
            step,
        }
    }
}

/// A payload consisting of a sequence of characters.
pub struct CharsSequencePayload<'a> {
    /// Value.
    pub value: Cow<'a, str>,
    /// Default value.
    pub default: Cow<'a, str>,
}

impl<'a> CharsSequencePayload<'a> {
    const fn new(value: Cow<'a, str>, default: Cow<'a, str>) -> Self {
        Self { value, default }
    }
}

/// A container for storing route parameter payloads.
pub struct ParametersPayloads(pub(crate) ToscaParametersPayloads<'static>);

impl ParametersPayloads {
    /// Retrieves the [`BoolPayload`] associated with the given parameter name.
    ///
    /// **It consumes the parameter.**
    ///
    /// # Errors
    ///
    /// An [`ErrorResponse`] is returned in the following cases:
    ///
    /// - When the given parameter is not found
    /// - When the given parameter has an incorrect type
    #[inline]
    pub fn bool(&mut self, name: &'static str) -> Result<BoolPayload, ErrorResponse> {
        self.insert(name, |payload| match (payload.value, payload.kind) {
            (ParameterValue::Bool(v), ParameterKind::Bool { default }) => {
                Ok(BoolPayload::new(v, default))
            }
            _ => Err(invalid_data(&format!("`{name}` is not a `bool` kind"))),
        })
    }

    /// Retrieves the [`U8Payload`] associated with the given parameter name.
    ///
    /// **It consumes the parameter.**
    ///
    /// # Errors
    ///
    /// An [`ErrorResponse`] is returned in the following cases:
    ///
    /// - When the given parameter is not found
    /// - When the given parameter has an incorrect type
    #[inline]
    pub fn u8(&mut self, name: &'static str) -> Result<U8Payload, ErrorResponse> {
        self.insert(name, |payload| match (payload.value, payload.kind) {
            (ParameterValue::U8(v), ParameterKind::U8 { default, min, max }) => {
                Ok(U8Payload::new(v, default, min, max))
            }
            _ => Err(invalid_data(&format!("`{name}` is not a `u8` kind"))),
        })
    }

    /// Retrieves the [`U16Payload`] associated with the given parameter name.
    ///
    /// **It consumes the parameter.**
    ///
    /// # Errors
    ///
    /// An [`ErrorResponse`] is returned in the following cases:
    ///
    /// - When the given parameter is not found
    /// - When the given parameter has an incorrect type
    #[inline]
    pub fn u16(&mut self, name: &'static str) -> Result<U16Payload, ErrorResponse> {
        self.insert(name, |payload| match (payload.value, payload.kind) {
            (ParameterValue::U16(v), ParameterKind::U16 { default, min, max }) => {
                Ok(U16Payload::new(v, default, min, max))
            }
            _ => Err(invalid_data(&format!("`{name}` is not a `u16` kind"))),
        })
    }

    /// Retrieves the [`U32Payload`] associated with the given parameter name.
    ///
    /// **It consumes the parameter.**
    ///
    /// # Errors
    ///
    /// An [`ErrorResponse`] is returned in the following cases:
    ///
    /// - When the given parameter is not found
    /// - When the given parameter has an incorrect type
    #[inline]
    pub fn u32(&mut self, name: &'static str) -> Result<U32Payload, ErrorResponse> {
        self.insert(name, |payload| match (payload.value, payload.kind) {
            (
                ParameterValue::U32(v),
                ParameterKind::U32 { default, min, max }
                | ParameterKind::RangeU32 {
                    default, min, max, ..
                },
            ) => Ok(U32Payload::new(v, default, min, max)),
            _ => Err(invalid_data(&format!("`{name}` is not a `u32` kind"))),
        })
    }

    /// Retrieves the [`U64Payload`] associated with the given parameter name.
    ///
    /// **It consumes the parameter.**
    ///
    /// # Errors
    ///
    /// An [`ErrorResponse`] is returned in the following cases:
    ///
    /// - When the given parameter is not found
    /// - When the given parameter has an incorrect type
    #[inline]
    pub fn u64(&mut self, name: &'static str) -> Result<U64Payload, ErrorResponse> {
        self.insert(name, |payload| match (payload.value, payload.kind) {
            (
                ParameterValue::U64(v),
                ParameterKind::U64 { default, min, max }
                | ParameterKind::RangeU64 {
                    default, min, max, ..
                },
            ) => Ok(U64Payload::new(v, default, min, max)),
            _ => Err(invalid_data(&format!("`{name}` is not a `u64` kind"))),
        })
    }

    /// Retrieves the [`F32Payload`] associated with the given parameter name.
    ///
    /// **It consumes the parameter.**
    ///
    /// # Errors
    ///
    /// An [`ErrorResponse`] is returned in the following cases:
    ///
    /// - When the given parameter is not found
    /// - When the given parameter has an incorrect type
    #[inline]
    pub fn f32(&mut self, name: &'static str) -> Result<F32Payload, ErrorResponse> {
        self.insert(name, |payload| match (payload.value, payload.kind) {
            (
                ParameterValue::F32(v),
                ParameterKind::F32 {
                    default,
                    min,
                    max,
                    step,
                },
            ) => Ok(F32Payload::new(v, default, min, max, step)),
            _ => Err(invalid_data(&format!("`{name}` is not a `f32` kind"))),
        })
    }

    /// Retrieves the [`F64Payload`] associated with the given parameter name.
    ///
    /// **It consumes the parameter.**
    ///
    /// # Errors
    ///
    /// An [`ErrorResponse`] is returned in the following cases:
    ///
    /// - When the given parameter is not found
    /// - When the given parameter has an incorrect type
    #[inline]
    pub fn f64(&mut self, name: &'static str) -> Result<F64Payload, ErrorResponse> {
        self.insert(name, |payload| match (payload.value, payload.kind) {
            (
                ParameterValue::F64(v),
                ParameterKind::F64 {
                    default,
                    min,
                    max,
                    step,
                }
                | ParameterKind::RangeF64 {
                    default,
                    min,
                    max,
                    step,
                },
            ) => Ok(F64Payload::new(v, default, min, max, step)),
            _ => Err(invalid_data(&format!("`{name}` is not a `f64` kind"))),
        })
    }

    /// Retrieves the [`CharsSequencePayload`] associated with
    /// the given parameter name.
    ///
    /// # Errors
    ///
    /// An [`ErrorResponse`] is returned in the following cases:
    ///
    /// - When the given parameter is not found
    /// - When the given parameter has an incorrect type
    #[inline]
    pub fn chars_sequence(
        &mut self,
        name: &'static str,
    ) -> Result<CharsSequencePayload<'_>, ErrorResponse> {
        self.insert(name, |payload| match (payload.value, payload.kind) {
            (ParameterValue::CharsSequence(s), ParameterKind::CharsSequence { default }) => {
                Ok(CharsSequencePayload::new(s, default))
            }
            _ => Err(invalid_data(&format!(
                "`{name}` is not a `characters sequence` kind"
            ))),
        })
    }

    #[inline]
    fn insert<T, F>(&mut self, name: &'static str, func: F) -> Result<T, ErrorResponse>
    where
        F: FnOnce(ParameterPayload) -> Result<T, ErrorResponse>,
    {
        let value = self
            .0
            .extract(name)
            .ok_or_else(|| invalid_data(&format!("`{name}` not found.")))?;

        func(value)
    }
}
