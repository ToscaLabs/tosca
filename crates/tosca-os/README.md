<div align="center">

# `tosca-os`

[![LICENSE][license badge]][license]

</div>

`tosca-os` is a library crate for building firmware for `tosca` devices
running on operating systems.

This crate targets devices that require more resources than typical embedded
systems, such as computing time, memory capacity, and interaction
with external components.

Currently, only firmware for `x86_64` and `ARM` architectures is supported
and covered by automated tests.

## Building

To build the crate with the `debug` profile, run:

```console
cargo build
```

To build with the `release` profile, which enables all time
and memory optimizations, run:

```console
cargo build --release
```

## Testing

To run the complete test suite:

```console
cargo test
```

## Features

The `stream` feature enables all data and methods necessary to
identify a multimedia stream sent from a device to a controller.

To disable all features, add the `--no-default-features` option to any of the
commands above.

## Building firmware examples

The [examples](https://github.com/ToscaLabs/tosca/tree/master/crates/tosca-os/examples)
directory contains firmware implemented using the `tosca-os` crate.
Each firmware is independent and can be moved to a separate repository by
simply replacing the `path` of `git` dependencies with those from `crates.io`.

## Statically-linked firmware binary

If a firmware cannot have system dependencies because it is a standalone
software, a statically-linked binary is required. To achieve this using the
`musl` toolchain, run the following command:

```bash
cargo build --manifest-path examples/firmware_device/Cargo.toml [--release] --target=x86_64-unknown-linux-musl
```

where `firmware_device` is the name of the example to build, and `--release`
is an optional argument that enables all time and memory optimizations.

## Cross-compiling to `aarch64` (ARM64) architecture

Install the [cross](https://github.com/cross-rs/cross) binary to easily
cross-compile Rust projects using Docker, without the need for custom
`Dockerfile`s.

```console
cargo install -f cross
```

To build a binary for an `ARM64` architecture, run:

```console
cd examples/firmware_device
cross build [--release] --target=aarch64-unknown-linux-musl
```

where `firmware_device` is the name of the example to build, and `--release`
is an optional argument that enables all time and memory optimizations.

<!-- Links -->
[license]: https://github.com/ToscaLabs/tosca?tab=readme-ov-file#license

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg
