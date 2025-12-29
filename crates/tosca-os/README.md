# `tosca-os`

[![LICENSE][license badge]][license]

This library crate provides a series of APIs to build a server which
represents the firmware of an IoT device running on an operating system.

Among its functionalities, it can interact send and receive data through
REST APIs.

The implemented devices can be found inside the [examples](./examples)
directory.

## Building complete firmware devices

The directory [examples](./examples) contains firmware implemented with
the `tosca-os` crate. Each firmware is independent from another one and it can
be moved in a separate repository.

Before any kind of build, run `cargo clean` to remove old builds configurations,
and then run `cargo update` to update all dependencies.

# Statically-linked firmware device

In order to build a statically-linked firmware, run the following command:

```bash
cargo build --manifest-path examples/firmware_device/Cargo.toml [--release] --target=x86_64-unknown-linux-musl
```

where `firmware_device` is the name of the example to build, while `--release`
is an optional argument which enables all time and memory optimizations.

# Cross-compilation to aarch64 (ARM64) architecture

Install a binary named [cross](https://github.com/cross-rs/cross) which allow
to easily cross-compile Rust projects using Docker, without messing with
custom `Dockerfile`s.

```console
cargo install -f cross
```

In order to build a binary for `ARM64` architecture run:

```console
cd examples/firmware_device
cross build [--release] --target=aarch64-unknown-linux-musl
```

where `firmware_device` is the name of the example to build, while `--release`
is an optional argument which enables all time and memory optimizations.

# Copy the cross-compiled binary to a board

To copy a cross-compiled binary to a board through `SSH`,
use the following command:

```console
scp -O target/aarch64-unknown-linux-musl/release/binary-name root@IPV4:~
```

where `IPV$` represents the address of the LAN interface which allows to connect
to board to a PC, while `~` represents the home directory on which the binary
will be copied.

Since `scp` is deprecated, and the version on a PC might use SFTP by default,
the `-O` flag reverts to the deprecated protocol.

<!-- Links -->
[license]: https://github.com/ToscaLabs/tosca?tab=readme-ov-file#license

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg
