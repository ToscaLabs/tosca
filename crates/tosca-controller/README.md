<div align="center">

# `tosca-controller`

[![LICENSE][license badge]][license]
[![Crates.io Version [crates.io badge]][crates.io]

</div>

The `tosca-controller` library crate offers a set of APIs to manage,
orchestrate, and interact with all `tosca`-compliant devices within a network.

A device is compliant with the `tosca` architecture if its firmware is built
using the `tosca` APIs designed for the relative microcontroller.

The core functionalities of this crate include:

- Discovering all devices within the network that are compliant with the
  `tosca` architecture
- Constructing and sending _REST_ requests to `tosca` devices to trigger
  one or more of their operations
- Defining security and privacy policies to allow or block requests
- Intercepting device events by subscribing to the brokers where
  they are published

To optimize system resource usage, `tosca-controller` leverages `tokio` as an
asynchronous executor. This improves performance by allowing concurrent
execution of independent tasks. If the underlying machine is multi-threaded,
the performance boost is further amplified, as tasks are distributed across
multiple threads too.

# Build Process

To compile this crate with the debug profile, run:

```console
cargo build
```

To compile this crate with the release profile, which applies all optimizations
for faster performance and reduced memory usage, run:

```
cargo build --release
```

# Build Examples

The [examples](https://github.com/ToscaLabs/tosca/tree/master/crates/tosca-controller/examples)
directory includes some examples to interact with the `tosca` devices within
a network. Each example is independent from another and can be moved to
a separate repository.

To build an example:

```console
cd examples/[example directory name]
cargo build [--release]
```

<!-- Links -->
[license]: https://github.com/ToscaLabs/tosca?tab=readme-ov-file#license
[crates.io]: https://crates.io/crates/tosca-controller

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg
[crates.io badge]: https://img.shields.io/crates/v/tosca-controller.svg
