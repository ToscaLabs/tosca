//! `tosca-drivers` is a library crate that provides architecture-agnostic
//! drivers for a variety of sensors and devices.
//!
//! All drivers are implemented using only the [`embedded-hal`] and
//! [`embedded-hal-async`] traits, ensuring compatibility with any platform
//! that supports these abstractions.
//!
//! [`embedded-hal`]: https://crates.io/crates/embedded-hal
//! [`embedded-hal-async`]: https://crates.io/crates/embedded-hal-async

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![no_std]

/// The `AM312` driver.
#[cfg(feature = "am312")]
pub mod am312;

/// The `BH1750` driver.
#[cfg(feature = "bh1750")]
pub mod bh1750;

/// The `DHT22` driver.
#[cfg(feature = "dht22")]
pub mod dht22;

/// The `DS18B20` driver.
#[cfg(feature = "ds18b20")]
pub mod ds18b20;
