//! `tosca-os` is a library crate for building firmware for `tosca` devices
//! running on operating systems.
//!
//! This crate targets devices that require more resources than typical embedded
//! systems, such as computing time, memory capacity, and interaction
//! with external components.
//!
//! Currently, only firmware for `x86_64` and `ARM` architectures is supported
//! and covered by automated tests.
//!
//! Device firmware consists of a description and a set of tasks, both exposed
//! through a client-server architecture in which the firmware operates as the
//! server and its controller as the client.
//!
//! A device description is defined as a sequence of fields, such as the
//! device name, the device kind, and other data used to establish a
//! connection with the a controller.
//!
//! When a controller makes a request to the firmware through route, the
//! firmware executes the corresponding task and sends a response! back to the
//! controller.
//! Routes may also accept parameters to configure tasks.
//!
//! Each route may have zero or more associated hazards, representing potential
//! risks during task execution. Even if no hazards are declared, a route may
//! still pose unknown risks to the device.
//! In such cases, the controller must decide whether to allow or block the
//! request based on its privacy policy.
//!
//! This crate **cannot** determine the outcome of device tasks at compile
//! time, as they depend on the runtime environment. Therefore, hazards
//! only informs a controller of the **possible** risks that might arise.
//!
//! An `std` environment is required to obtain full crate functionality.

#![deny(unsafe_code)]
#![deny(missing_docs)]

/// All supported device types.
pub mod devices;

/// General device definition along with its methods.
pub mod device;
/// Error management.
pub mod error;
/// All responses kinds along with their payloads.
pub mod responses;
/// The firmware server.
pub mod server;
/// The discovery service used to make the firmware detectable on the network.
pub mod service {
    pub use super::services::{ServiceConfig, TransportProtocol};
}

/// Utilities for parsing request parameters and constructing responses.
pub mod extract {
    pub use axum::extract::{FromRef, Json, Path, State};
    pub use axum::http::header;
}

mod mac;
mod services;
