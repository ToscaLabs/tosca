//! `tosca-esp32c3` is a library crate for building firmware for `tosca` devices
//! using an `ESP32-C3` microcontroller.
//!
//! It provides APIs to:
//!
//! - Connect a device to a `Wi-Fi` access point
//! - Build the network stack
//! - Configure the `mDNS-SD` discovery service
//! - Define events for specific route tasks
//! - Initialize and run an `HTTP` server
//!
//! The device APIs are designed to guide developers in defining their own
//! devices, aiming to minimize the ambiguities that could arise during
//! firmware development.
//!
//! Device firmware consists of a description and a set of tasks, both exposed
//! through a client-server architecture in which the firmware operates as the
//! server and its controller as the client.
//!
//! A device description is defined as a sequence of fields, such as the
//! device name, the device kind, and other data used to establish a
//! connection with a controller.
//!
//! When a controller makes a request to the firmware through a route, the
//! firmware executes the corresponding task and sends a response! back to the
//! controller.
//! Routes may also accept parameters to configure tasks
//!
//! An event can be associated with a route to monitor data produced by a
//! sensor. Both integer and floating-point values are supported, as well as
//! events triggered by changes in the device state configuration.
//!
//! Each route can define zero or more associated hazards, representing
//! potential risks during task execution. Even if no hazards are declared,
//! a route may still pose unknown risks to the device.
//! In such cases, the controller must decide whether to allow or block the
//! request based on its privacy policy.
//!
//! This crate **cannot** determine the outcome of device tasks at compile
//! time, as they depend on the runtime environment. Therefore, hazards
//! only informs a controller of the **possible** risks that might arise.

#![no_std]
#![deny(missing_docs)]

extern crate alloc;

/// All supported device types.
pub mod devices;

/// General device definition along with its methods.
pub mod device;
/// Error management.
pub mod error;
/// Events and their data.
pub mod events;
/// The `mDNS-SD` discovery service.
pub mod mdns;
/// The network stack builder.
pub mod net;
/// All route parameters.
pub mod parameters;
/// All responses kinds along with their payloads.
pub mod response;
/// The firmware server.
pub mod server;
/// The device state.
pub mod state;
/// The `Wi-Fi` controller.
pub mod wifi;

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write($val);
        x
    }};
}

pub(crate) use mk_static;
