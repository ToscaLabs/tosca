<div align="center">

# `tosca-drivers`

[![LICENSE][license badge]][license]
[![Crates.io Version [crates.io badge]][crates.io]

</div>

A Rust library crate providing architecture-agnostic drivers for various sensors
and devices.

This crate currently includes drivers for:

- [**AM312**](https://github.com/ToscaLabs/tosca/blob/master/crates/tosca-drivers/docs/am312.md): PIR motion sensor.
- [**BH1750**](https://github.com/ToscaLabs/tosca/blob/master/crates/tosca-drivers/docs/bh1750.md): ambient light sensor.
- [**DHT22**](https://github.com/ToscaLabs/tosca/blob/master/crates/tosca-drivers/docs/dht22.md): temperature and humidity sensor.
- [**DS18B20**](https://github.com/ToscaLabs/tosca/blob/master/crates/tosca-drivers/docs/ds18b20.md): temperature sensor.

All drivers are implemented using only the [`embedded-hal`] and
[`embedded-hal-async`] traits, making them compatible with any platform that
supports these abstractions.

For each driver, a short documentation is provided containing a description and
the wiring diagram in [docs](https://github.com/ToscaLabs/tosca/tree/master/crates/tosca-drivers/docs).
These diagrams were created using [`Fritzing`], an open-source tool,
and the corresponding project files can be found in
[fritzing](https://github.com/ToscaLabs/tosca/tree/master/crates/tosca-drivers/docs/fritzing).

## Features

You can enable only the sensors you need using Cargo features:

```toml
[dependencies]
tosca-drivers.version = "0.1.0"
tosca-drivers.default-features = false
tosca-drivers.features = ["bh1750", "dht22"] # only include needed drivers
```

<!-- Links -->
[license]: https://github.com/ToscaLabs/tosca?tab=readme-ov-file#license
[crates.io]: https://crates.io/crates/tosca-drivers
[`embedded-hal`]: https://crates.io/crates/embedded-hal
[`embedded-hal-async`]: https://crates.io/crates/embedded-hal-async
[`Fritzing`]: https://fritzing.org/download/

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg
[crates.io badge]: https://img.shields.io/crates/v/tosca-drivers.svg
