<div align="center">

# `tosca`

[![LICENSE][license badge]][license]

</div>

This library acts as an interface between a device and a controller.
All other `tosca` crates must incorporate it in some way into their API
definition.

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

To ensure compatibility with embedded devices, this library is `no_std`, linking
to the `core` crate instead of the `std` crate.

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

To reduce the final binary size and speed up compilation, several features have
been added.
The `stream` feature enables all data and methods necessary to
identify a multimedia stream sent from a device to a controller.
The `deserialize` feature enables data deserialization, generally
useful for controllers but not for devices, which typically handle only
serialization.

To disable all features, add the `--no-default-features` option to any of the
commands above.

<!-- Links -->
[license]: https://github.com/ToscaLabs/tosca?tab=readme-ov-file#license

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg
