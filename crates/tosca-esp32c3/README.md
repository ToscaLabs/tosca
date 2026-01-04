<div align="center">

# `tosca-esp32c3`

[![LICENSE][license badge]][license]
[![Crates.io Version [crates.io badge]][crates.io]

</div>

A library crate for building firmware for `tosca` devices using an `ESP32-C3`
microcontroller.

It provides APIs to:

- Connect a device to a `Wi-Fi` access point
- Build the network stack
- Configure the `mDNS-SD` discovery service
- Define events for specific route tasks
- Initialize and run an `HTTP` server

The device APIs are designed to guide developers in defining their own
devices, aiming to minimize the ambiguities that could arise during
firmware development.

Currently, only a light device is implemented as a device.
However, this does not prevent the addition of other devices without altering
the overall crate structure.

## Build Process

To compile this crate with the `debug` profile, run:

```console
cargo build
```

To compile this crate with the `release` profile, which applies all
optimizations for faster performance and reduced memory usage, run:

```console
cargo build --release
```

## Build process for firmware devices

The [examples](https://github.com/ToscaLabs/tosca/tree/master/crates/tosca-esp32c3/examples)
directory includes firmware examples built with the `tosca-esp32c3` crate.
Each example is independent from another and can be moved to a separate
repository.

To build a firmware run:

```console
cd examples/[firmware directory name]
cargo build
```

To flash and run the firmware on an `ESP32-C3` board:

```console
cd examples/[firmware directory name]
cargo run --release
```

> [!IMPORTANT]
> Always use the release profile `[--release]` when building esp-hal.
  The dev profile can potentially be one or more orders of magnitude
  slower than release profile, and may cause issues with timing-sensitive
  peripherals and/or devices.

<!-- Links -->
[license]: https://github.com/ToscaLabs/tosca?tab=readme-ov-file#license
[crates.io]: https://crates.io/crates/tosca-esp32c3

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg
[crates.io badge]: https://img.shields.io/crates/v/tosca-esp32c3.svg
