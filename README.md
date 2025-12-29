<div align="center">

# `tosca`

[![Actions][actions badge]][actions]
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

- **Versatile**: On one hand, the crate offers APIs to build firmware for
  microcontrollers with various hardware architectures, supporting both
  bare-metal and OS-based devices.
  On the other hand, it supplies APIs for creating software that interacts
  with these devices.

- **Customizable**: Most of the APIs are designed as a sequence of blocks, where
  each block represents a single feature or a set of features that can be easily
  added or removed by simply adding or deleting lines of code. As an example, if
  your device supports events, you simply need to add the event APIs to your
  firmware server to send the data to its controller. You do not have to touch
  those APIs if your firmware do not use events.

- **Secure**: Written in [Rust](https://rust-lang.org/), a language known for
  its focus on performance and reliability. Its rich type system and ownership
  model guarantees memory safety and thread safety, eliminating many classes of
  bugs at compile-time.

## Framework Structure

The main crate is [tosca](./crates/tosca):
a library that acts as an interface between a device and a controller.
All other `tosca` crates must incorporate it in some way into their API
definition.

It can:

- Create and manage **REST** routes to issue commands from a
  controller to a device.
- Describe a device, including the structure of its firmware, its internal data
  and methods, as well as information about its resource consumption at the
  economic and energy levels.
- Associate hazards with a route to describe the risks of a device
  operation.

To ensure compatibility with embedded devices, this library is `no_std`, thus
linking to the `core`-crate instead of the `std`-crate.

The [tosca-os](./crates/tosca-os) and [tosca-esp32c3](./crates/tosca-esp32c3)
are two Rust libraries crates for building firmware. They integrate the `tosca`
library as a dependency in their APIs to share a common interface.

The `tosca-os` library crate is designed for firmware that runs on operating
systems.
In the [tosca-os/examples](./crates/tosca-os/examples) directory, you can find
simple examples of [light](./crates/tosca-os/examples/light) and
[ip-camera](./crates/tosca-os/examples/ip-camera) firmware.

The `tosca-esp32c3` library crate is designed for firmware that runs on
`ESP32-C3` microcontrollers.
In the [tosca-esp32c3/examples](./crates/tosca-esp32c3/examples) directory,
you can find several **light** firmware examples showcasing different features.

The [tosca-drivers](./crates/tosca-drivers) library crate provides
architecture-agnostic drivers for a pool of sensors and devices.
All drivers are built on top of [`embedded-hal`] and [`embedded-hal-async`],
ensuring compatibility across all supported hardware platforms.

The [tosca-controller](./crates/tosca-controller) library crate defines
a set of APIs to manage, orchestrate, and interact with firmware built using
the crates mentioned above. In the
[tosca-controller/examples](./crates/tosca-controller/examples) directory,
you can find some examples demonstrating various methods for receiving events
from devices.

## Building

This repository is a Cargo workspace composed of several crates. Dependencies
common to all crates are defined in the root `Cargo.toml`, ensuring they are
compiled once and their resulting binaries shared across all crates.
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

To build only a specific crate, navigate to its corresponding subdirectory
within the [crates](./crates) directory and run the same build commands as
described above.

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

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.

## Contribution

Contributions are welcome via pull request.
The [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct)
applies.

Unless explicitly stated otherwise, all contributions will be licensed under
the project defined licenses, without any additional terms or conditions.

<!-- Links -->
[actions]: https://github.com/ToscaLabs/tosca/actions
[codecov]: https://codecov.io/gh/ToscaLabs/tosca
[license]: https://github.com/ToscaLabs/tosca?tab=readme-ov-file#license
[`embedded-hal`]: https://crates.io/crates/embedded-hal
[`embedded-hal-async`]: https://crates.io/crates/embedded-hal-async

<!-- Badges -->
[actions badge]: https://github.com/ToscaLabs/tosca/workflows/ci/badge.svg
[codecov badge]: https://codecov.io/gh/ToscaLabs/tosca/branch/master/graph/badge.svg
[license badge]: https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg
