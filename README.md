<div align="center">

# `tosca`

[![Actions][actions badge]][actions]
[![Crates.io Version [crates.io badge]][crates.io]
[![Minimum Supported Rust Version][msrv badge]][msrv]
[![Codecov][codecov badge]][codecov]
[![LICENSE][license badge]][license]

</div>

> [!CAUTION]
> The `tosca` framework is in a very early, experimental stage of development.
> The APIs are still unstable and subject to change.
> Be aware that even a minor version may introduce API breakages.
> A major version will be released only when the APIs remain stable and
unchanged for an extended period.
> This approach aims to provide clearer and more precise APIs, shaped by user
feedback and suggestions during the initial stages of the project.

`tosca` is a versatile, customizable, and secure IoT framework.

- **Versatile**: The framework offers APIs to develop firmware for various
  hardware architectures, supporting both bare-metal and OS-based devices.
  At the same time, it also supplies APIs for creating software that interacts
  with the firmware of these devices.

- **Customizable**: Most of the APIs are designed as a sequence of code blocks,
  where each block represents a single feature or a set of features. These
  blocks can be combined by adding or removing lines of code. For example, if
  your device supports events, you only need to integrate the event APIs into
  your firmware server to send the data to its controller. If your firmware does
  not use events, there is no need to touch those APIs.

- **Secure**: Written in [Rust](https://rust-lang.org/), a language renowned
  for its emphasis on performance and reliability. Its rich type system and
  ownership  model guarantees memory and thread safety, preventing many classes
  of bugs at compile-time.

## Framework Structure

The framework revolves around the `tosca` interface, which connects two sides
of the framework. The first is the _Firmware Side_, responsible for developing
firmware and providing drivers for sensors, while the second is the
_Controller Side_, responsible for interacting with the devices built using
the `tosca` framework.

### `tosca` Interface

[tosca](https://github.com/ToscaLabs/tosca/tree/master/crates/tosca) is the main
crate of the framework. It serves as an interface between a device and
a controller.

It can:

- Create and manage **REST** routes to issue commands from a
  controller to a device. Each route can even define parameters that mirror
  those used by a device in its operations. The responses to a route can include
  a simple `Ok` indicating success on the device side, a `Serial`
  response with additional data about the device operation, and an `Info`
  response containing metadata and other details about the device. The `Stream`
  response is optional and can be enabled via a feature, delivering chunks of
  multimedia data as bytes.
- Describe a device, including the structure of its firmware, its internal data
  and methods, as well as information about its resource consumption at the
  economic and energy levels.
- Associate hazards with a route to describe the risks of a device
  operation. A hazard is categorized into three types: _Safety_,
  _Financial_, or _Privacy_. The _Safety_ category covers risks to human life,
  the _Financial_ category addresses the economic impacts, and the _Privacy_
  category relates to issues concerning data management.

It offers several features that reduce the final binary size and speed up
compilation.
The `stream` feature enables all data and methods necessary to identify a
multimedia stream sent from a device to a controller.
The `deserialize` feature enables data deserialization, which is generally
useful for controllers but not for devices, as they typically handle only
serialization.

To ensure compatibility with embedded devices, this library is `no_std`, linking
to the `core` crate instead of the `std` crate.

### Firmware Side

The [tosca-os](https://github.com/ToscaLabs/tosca/tree/master/crates/tosca-os)
and
[tosca-esp32c3](https://github.com/ToscaLabs/tosca/tree/master/crates/tosca-esp32c3)
crates are two libraries used for building firmware. As previously stated, they
integrate the `tosca` library as a dependency in their APIs to maintain a
common interface.

The `tosca-os` library crate is designed for firmware running on operating
systems.
In the [tosca-os/examples](https://github.com/ToscaLabs/tosca/tree/master/crates/tosca-os/examples) directory, you will find a simple example of both light and IP-camera firmware.

The `tosca-esp32c3` library crate is designed for firmware running on
`ESP32-C3` microcontrollers.
In the [tosca-esp32c3/examples](https://github.com/ToscaLabs/tosca/tree/master/crates/tosca-esp32c3/examples) directory, you will find several **light** firmware examples
demonstrating various features of this library.

The [tosca-drivers](https://github.com/ToscaLabs/tosca/tree/master/crates/tosca-drivers)
library crate provides architecture-agnostic drivers for a range of sensors
and devices. All drivers are built on top of [`embedded-hal`] and
[`embedded-hal-async`], ensuring compatibility across all supported
hardware platforms.

### Controller Side

The [tosca-controller](https://github.com/ToscaLabs/tosca/tree/master/crates/tosca-controller)
library crate defines a set of APIs for managing, orchestrating, and interacting
with firmware built using the crates mentioned above. In the
[tosca-controller/examples](https://github.com/ToscaLabs/tosca/tree/master/crates/tosca-controller/examples) directory, you will find some examples demonstrating various methods
for receiving events from devices.

## Building

The framework [repository](https://github.com/ToscaLabs/tosca) is a Cargo
workspace composed of several crates. Dependencies common to all crates are
defined in the root `Cargo.toml`, ensuring they are compiled once and their
resulting binaries shared across all crates.
The same approach is applied to the `tosca` metadata.

To build the entire workspace with the `debug` profile from the root of the
repository, run:

```console
cargo build
```

To build the entire workspace with the `release` profile, which enables all time
and memory optimizations, run the following command from the root of the
repository:

```console
cargo build --release
```

To build a specific crate, navigate to its corresponding subdirectory
within the [crates](https://github.com/ToscaLabs/tosca/tree/master/crates)
directory and run the same build commands as described above.

If a crate provides features that you want to disable, add the
`--no-default-features` option to the commands above.

> [!NOTE]
> The `tosca-esp32c3` library crate is not part of the workspace and must be
built separately, as it targets a specific architecture
(`riscv32imc-unknown-none-elf`), requiring a specialized build process.
The [per-package-target](https://doc.rust-lang.org/cargo/reference/unstable.html#per-package-target)
feature in Cargo is unstable and only available on the nightly toolchain.

## Testing

To run the full test suite for each crate, execute the following command:

```console
cargo test
```

This may take several minutes, depending on the tests defined in each crate.

If only the tests for a specific crate need to be run, navigate to the
corresponding crate subdirectory and execute the `cargo test` command.

If a crate provides features that you want to disable, add the
`--no-default-features` option to the commands above.

## License

Licensed under either of

- [Apache License, Version 2.0](https://github.com/ToscaLabs/tosca/blob/master/LICENSE-APACHE)
- [MIT License](https://github.com/ToscaLabs/tosca/blob/master/LICENSE-MIT)

at your option.

## Contribution

Contributions are welcome via pull request.
The [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct)
applies.

Unless explicitly stated otherwise, all contributions will be licensed under
the project defined licenses, without any additional terms or conditions.

<!-- Links -->
[actions]: https://github.com/ToscaLabs/tosca/actions
[crates.io]: https://crates.io/crates/tosca
[msrv]: https://crates.io/crates/tosca
[codecov]: https://codecov.io/gh/ToscaLabs/tosca
[license]: https://github.com/ToscaLabs/tosca?tab=readme-ov-file#license
[`embedded-hal`]: https://crates.io/crates/embedded-hal
[`embedded-hal-async`]: https://crates.io/crates/embedded-hal-async

<!-- Badges -->
[actions badge]: https://github.com/ToscaLabs/tosca/workflows/ci/badge.svg
[crates.io badge]: https://img.shields.io/crates/v/tosca.svg
[msrv badge]: https://img.shields.io/crates/msrv/tosca
[codecov badge]: https://codecov.io/gh/ToscaLabs/tosca/branch/master/graph/badge.svg
[license badge]: https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg
