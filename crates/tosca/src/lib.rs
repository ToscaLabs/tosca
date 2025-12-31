//! The communication interface among a `tosca` device and its controller.
//!
//! This crate provides APIs to:
//!
//! - Encode and decode information about a device structure and its routes.
//!   A route is a path that allows a controller to start one or
//!   more tasks on a device. Each route is always associated with a response.
//! - Manage hazards that may arise during the execution of a route.
//!   Hazards describe safety, privacy, and financial risks, and are always
//!   associated with a route.
//! - Manage route parameters. Parameters may represent external
//!   information required for device tasks or conditions that control whether
//!   instructions are executed. For example, a boolean parameter might control
//!   the on/off state of a light, while a float range might adjust its
//!   brightness state.
//!
//! Data exchange between the device and controller requires structures to be
//! serializable and deserializable. A device serializes these structures
//! while the controller deserializes them and uses the data for its tasks.
//! A device can avoid importing deserialization functions by disabling the
//! `deserialize` feature at compile time.
//!
//! This crate can be compiled for both `std` and `no_std` environments.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![no_std]

extern crate alloc;

mod macros;

/// Description of a device and its associated routes.
pub mod device;
/// Economic information about a device.
pub mod economy;
/// Energy-related information about a device.
pub mod energy;
/// Event descriptions and methods.
pub mod events;
/// Hazard descriptions and methods.
pub mod hazards;
/// Route parameters.
pub mod parameters;
/// All supported responses from a device.
pub mod response;
/// Definition of device routes.
pub mod route;

#[cfg(test)]
#[cfg(feature = "deserialize")]
pub(crate) fn serialize<T: serde::Serialize>(value: T) -> serde_json::Value {
    serde_json::to_value(value).unwrap()
}

#[cfg(test)]
#[cfg(feature = "deserialize")]
pub(crate) fn deserialize<T: serde::de::DeserializeOwned>(value: serde_json::Value) -> T {
    serde_json::from_value(value).unwrap()
}
