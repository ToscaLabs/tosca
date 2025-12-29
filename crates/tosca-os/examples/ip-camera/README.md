# Ip-Camera Server

[![LICENSE][license badge]][license]

An Ip-Camera server usable from different operating systems for taking
screenshots and sending a video stream of the surrounding environment.

# Building

To build the camera binary, run the command:

```console
cargo build [--release]
```

where `--release` is an option which enables all time and memory optimizations
needed for having a binary usable in production. If the option is not inserted,
the binary will be built with all debug symbols inside.

## Cross-compilation to aarch64 (ARM64) architecture

Install a binary named [cross](https://github.com/cross-rs/cross) which allow
to easily cross-compile Rust projects using Docker, without messing with
custom `Dockerfile`s.

```console
cargo install -f cross
```

To build a binary for an `ARM64` architecture run:

```console
cross build [--release] --target=aarch64-unknown-linux-musl
```

where `--release` is an option which enables all time and memory optimizations
needed for having a binary usable in production. If the option is not inserted,
the binary will be built with all debug symbols inside.

To build a binary for a `RISCV64GC` architecture run:

```console
cross build [--release] --target=riscv64gc-unknown-linux-gnu
```

This build is based on `gnu` since `musl` is not supported yet. Therefore, the
usage on an embedded system is limited.

# Usage

To run the Ip-camera, the command is:

```console
cargo run -- --hostname your-hostname
```

where `your-hostname` is the mandatory parameters to define a specific hostname
to contact the Ip-camera. An example of a complete hostname, leaving all the
other parameters untouched is: `http://your-hostname.local:3000`.

To issue commands to the Ip-Camera, it is necessary to look at routes manual
printed on the console at the Ip-camera startup.

For example, to start the streaming, one must write
`http://your-hostname.local:3000/camera/stream` on the browser.

<!-- Links -->
[license]: https://github.com/ToscaLabs/tosca?tab=readme-ov-file#license

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg
