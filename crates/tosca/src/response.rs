use alloc::borrow::Cow;

use serde::Serialize;

use crate::device::DeviceInfo;

/// The header value associated with a response sent by a device which had
/// failed to serialize its values.
///
/// This constant signals to the controller to discard the invalid response
/// because a serialization error occurred on a device.
pub const SERIALIZATION_ERROR: &str = "Serialization-Error";

/// Response kinds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub enum ResponseKind {
    /// This response transmits a concise JSON message over the network to
    /// notify a controller that an operation completed successfully.
    #[default]
    Ok,
    /// This response transmits a JSON message over the network containing
    /// the data produced during a device operation.
    Serial,
    /// This response transmits a JSON message over the network containing
    /// a device energy and economy information.
    Info,
    /// This response transmits a stream of data, represented as a
    /// sequence of bytes, over the network.
    #[cfg(feature = "stream")]
    Stream,
}

impl core::fmt::Display for ResponseKind {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Ok => "Ok",
            Self::Serial => "Serial",
            Self::Info => "Info",
            #[cfg(feature = "stream")]
            Self::Stream => "Stream",
        }
        .fmt(f)
    }
}

/// A response which transmits a concise JSON message over the network to notify
/// a controller that an operation completed successfully.
#[derive(Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub struct OkResponse {
    action_terminated_correctly: bool,
}

impl OkResponse {
    /// Generates an [`OkResponse`].
    #[must_use]
    pub const fn ok() -> Self {
        Self {
            action_terminated_correctly: true,
        }
    }
}

/// A response which transmits a JSON message over the network containing
/// the data produced during a device operation.
#[derive(Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub struct SerialResponse<T: Serialize>(T);

impl<T: Serialize> SerialResponse<T> {
    /// Generates a [`SerialResponse`].
    #[must_use]
    pub const fn new(data: T) -> Self {
        Self(data)
    }
}

/// A response which transmits a JSON message over the network containing
/// a device's energy and economy information.
#[derive(Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub struct InfoResponse(DeviceInfo);

impl InfoResponse {
    /// Generates an [`InfoResponse`].
    #[must_use]
    pub const fn new(info: DeviceInfo) -> Self {
        Self(info)
    }
}

/// All possible errors that may cause a device operation to fail.
#[derive(Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub enum ErrorKind {
    /// Some data encountered during a device operation is invalid or malformed.
    InvalidData,
    /// An internal error has occurred during the execution of a device
    /// operation.
    Internal,
}

/// A response providing details about an error encountered during a
/// device operation.
///
/// Contains the [`ErrorKind`], a general error description,
/// and optional information about the encountered error.
#[derive(Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub struct ErrorResponse<'a> {
    /// Error kind.
    pub error: ErrorKind,
    /// Error description.
    pub description: Cow<'a, str>,
    /// Information describing the encountered error.
    pub info: Option<Cow<'a, str>>,
}

impl<'a> ErrorResponse<'a> {
    /// Generates an [`ErrorResponse`].
    ///
    /// Requires specifying the [`ErrorKind`] and a general error description.
    #[must_use]
    #[inline]
    pub fn with_description(error: ErrorKind, description: &'a str) -> Self {
        Self {
            error,
            description: Cow::Borrowed(description),
            info: None,
        }
    }

    /// Generates an [`ErrorResponse`].
    ///
    /// Requires specifying the [`ErrorKind`], a general error
    /// description, and optional information about the encountered error.
    #[must_use]
    #[inline]
    pub fn with_description_error(error: ErrorKind, description: &'a str, info: &'a str) -> Self {
        Self {
            error,
            description: Cow::Borrowed(description),
            info: Some(Cow::Borrowed(info)),
        }
    }

    /// Generates an [`ErrorResponse`] for invalid data.
    ///
    /// Requires specifying a general error description.
    #[must_use]
    #[inline]
    pub fn invalid_data(description: &'a str) -> Self {
        Self::with_description(ErrorKind::InvalidData, description)
    }

    /// Generates an [`ErrorResponse`] for invalid data.
    ///
    ///
    /// Requires specifying a general error description and optional
    /// information about the encountered error.
    #[must_use]
    #[inline]
    pub fn invalid_data_with_error(description: &'a str, info: &'a str) -> Self {
        Self::with_description_error(ErrorKind::InvalidData, description, info)
    }

    /// Generates an [`ErrorResponse`] for an internal error.
    ///
    /// Requires specifying a general error description.
    #[must_use]
    #[inline]
    pub fn internal(description: &'a str) -> Self {
        Self::with_description(ErrorKind::Internal, description)
    }

    /// Generates an [`ErrorResponse`] for an internal error.
    ///
    ///
    /// Requires specifying a general error description and optional
    /// information about the encountered error.
    #[must_use]
    #[inline]
    pub fn internal_with_error(description: &'a str, info: &'a str) -> Self {
        Self::with_description_error(ErrorKind::Internal, description, info)
    }
}

#[cfg(test)]
#[cfg(feature = "deserialize")]
mod tests {
    use serde::Deserialize;

    use crate::{deserialize, serialize};

    use super::{OkResponse, SerialResponse, Serialize};

    use super::{Cow, DeviceInfo, ErrorKind, ErrorResponse, InfoResponse};

    #[test]
    fn test_ok_response() {
        assert_eq!(
            deserialize::<OkResponse>(serialize(OkResponse::ok())),
            OkResponse {
                action_terminated_correctly: true,
            }
        );
    }

    #[test]
    fn test_serial_value_response() {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct SerialValue {
            value: u32,
        }

        assert_eq!(
            deserialize::<SerialValue>(serialize(SerialResponse::new(SerialValue { value: 42 }))),
            SerialValue { value: 42 },
        );
    }

    #[test]
    fn test_serial_cow_response() {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct SerialCow<'a> {
            value: Cow<'a, str>,
        }

        assert_eq!(
            deserialize::<SerialCow>(serialize(SerialResponse::new(SerialCow {
                value: Cow::Borrowed("hi")
            }))),
            SerialCow {
                value: Cow::Owned("hi".into())
            },
        );
    }

    #[test]
    fn test_info_response() {
        let energy = crate::energy::Energy::init_with_water_use_efficiency(
            crate::energy::WaterUseEfficiency::init_with_gpp(42.0),
        );

        assert_eq!(
            deserialize::<DeviceInfo>(serialize(InfoResponse::new(
                DeviceInfo::empty().add_energy(energy)
            ))),
            DeviceInfo {
                energy: crate::energy::Energy {
                    energy_efficiencies: None,
                    carbon_footprints: None,
                    water_use_efficiency: Some(crate::energy::WaterUseEfficiency {
                        gpp: Some(42.0),
                        penman_monteith_equation: None,
                        wer: None,
                    }),
                },
                economy: crate::economy::Economy::empty(),
            }
        );
    }

    #[test]
    fn test_error_response() {
        let error = ErrorResponse::with_description(
            ErrorKind::InvalidData,
            "Invalid data error description",
        );

        assert_eq!(
            deserialize::<ErrorResponse>(serialize(error)),
            ErrorResponse {
                error: ErrorKind::InvalidData,
                description: Cow::Borrowed("Invalid data error description"),
                info: None,
            }
        );
    }
}
