<div align="center">

# `tosca-esp32c3`

[![LICENSE][license badge]][license]

</div>

A Rust library crate designed to develop `Tosca` firmware on `ESP32-C3`
boards.

It offers APIs to:

- Connect a device to a `Wi-Fi` access point
- Build the network stack
- Configure the `mDNS-SD` discovery service
- Initialize and run an `HTTP` server

The device APIs have been conceived to assist developers in defining their
own devices, minimizing as much as possible the ambiguities that may arise
during firmware development.

Some of the most common errors include:

- Absence of the fundamental methods that define a device
- Missing or incorrect hazard information for an operation

To ensure device customization, there are also APIs to add
device-specific operations. For example, an RGB light may require methods
to control its colors individually, a feature that is not necessary for a
basic light.

Currently, only the [light](./src/devices/light.rs) device and its associated
APIs have been implemented within the [src/devices](./src/devices) directory.
However, this does not prevent other devices from being implemented without
needing to change the overall crate structure.

Multiple implementations of real devices integrated with different sensors can
be found in the [examples](./examples) directory.

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
> Always use the release profile [--release] when building esp-hal.
  The dev profile can potentially be one or more orders of magnitude
  slower than release profile, and may cause issues with timing-senstive
  peripherals and/or devices.

<!-- Links -->
[license]: https://github.com/ToscaLabs/tosca?tab=readme-ov-file#license

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg
