//! The `tosca-controller` library crate provides a set of APIs for managing,
//! orchestrating, and interacting with all `tosca` devices within a
//! network.
//!
//! A device is considered compliant with `tosca` if its firmware is built
//! using the APIs of the `tosca` framework designed for the device's underlying
//! hardware architecture.
//!
//! Core functionalities of this crate include:
//!
//! - Discovering all devices within the network that are compliant with the
//!   `tosca` architecture
//! - Constructing and sending `REST` requests to `tosca` devices to trigger
//!   their tasks
//! - Defining privacy policies to allow or block requests to a device
//! - Intercepting device events by subscribing to the brokers where
//!   they are published
//!
//! To optimize system resource usage, `tosca-controller` leverages `tokio` as
//! an asynchronous executor, allowing concurrent execution of independent
//! tasks. This approach improves performance, especially when running on
//! multi-threaded systems, where tasks are distributed across
//! multiple threads for additional efficiency.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// A controller for interacting with `tosca` devices.
pub mod controller;
/// Device data along with its associated methods.
pub mod device;
/// A service for discovering all `tosca` devices within a network.
pub mod discovery;
/// Error management.
pub mod error;
/// All events data.
pub mod events;
/// A privacy policy manager that blocks or allows the requests to devices
/// based on a set of privacy rules.
pub mod policy;
/// Request data and the associated methods.
pub mod request;
/// All supported methods and data for handling `tosca` device responses.
pub mod response;

#[cfg(test)]
mod tests;
