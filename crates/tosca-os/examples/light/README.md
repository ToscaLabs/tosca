# Light

[![LICENSE][license badge]][license]

## Building

Run the command

```console
cargo build
```

## Running the server

The server runs on `localhost` and listens to port `3000`. To make it run:

```console
cargo run
```

## REST API examples

Through `curl` or a web browser, it is possible to call the APIs which perform
some actions on a device.

**Turn a light on**

```console
curl -X PUT 127.0.0.1:3000/light/on/4.0/true
```

```console
curl -X POST 127.0.0.1:3000/light/on/4.0/true
```

**Turn a light off**

```console
curl -X PUT 127.0.0.1:3000/light/off
```

**Toggle a light**

```console
curl -X PUT 127.0.0.1:3000/light/toggle
```

At server startup, an initial message signalling its effective execution
is printed.

```
Starting server...
```

<!-- Links -->
[license]: https://github.com/ToscaLabs/tosca?tab=readme-ov-file#license

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg
