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
feedback and suggestions during the initial stages of the project

`tosca` is an IoT framework thought to be secure, flexible and customizable.

On one hand, it provides a set of APIs to build firmware for microcontrollers
with various hardware architectures, while on the other hand, it offers APIs
for developing software to interact with those devices.

From a structural perspective, this repository is a Cargo workspace
made up of several crates.

The main one is [tosca](./crates/tosca), a Rust library crate that serves as the
primary interface for all the other crates. It can:

- Create and manage **REST** routes, including their route parameters
- Describe a device, including its structure, internal data, and methods
- Associate hazards with a device

To ensure compatibility with embedded devices, this library is `no_std`, so
it links to the `core`-crate instead of the `std`-crate.

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

Licensed under

- [MIT License](LICENSE)

## Contribution

Contributions are welcome via pull request.
The [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct) 
applies.

Unless explicitly stated otherwise, all contributions will be licensed under
the project defined license, without any additional terms or conditions.

<!-- Links -->
[actions]: https://github.com/ToscaLabs/tosca/actions
[codecov]: https://codecov.io/gh/ToscaLabs/tosca
[license]: https://github.com/ToscaLabs/tosca/blob/master/LICENSE
[`embedded-hal`]: https://crates.io/crates/embedded-hal
[`embedded-hal-async`]: https://crates.io/crates/embedded-hal-async

<!-- Badges -->
[actions badge]: https://github.com/ToscaLabs/tosca/workflows/ci/badge.svg
[codecov badge]: https://codecov.io/gh/ToscaLabs/tosca/branch/master/graph/badge.svg
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg
