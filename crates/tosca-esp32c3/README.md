<div align="center">

# `tosca-esp32c3`

[![LICENSE][license badge]][license]

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

Currently, only the [light](./src/devices/light.rs) device and its associated
APIs are implemented within the [src/devices](./src/devices) directory.
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

The [examples](./examples) directory includes firmware examples built with the
`tosca-esp32c3` crate. Each example is independent from another and can be moved
to a separate repository.

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
  slower than release profile, and may cause issues with timing-senstive
  peripherals and/or devices.

<!-- Links -->
[license]: https://github.com/ToscaLabs/tosca?tab=readme-ov-file#license

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg
