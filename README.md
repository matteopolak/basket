# Basket

[![cli animation](docs/cli.gif)](docs/cli.tape)

[![test status](https://github.com/matteopolak/basket/actions/workflows/ci.yml/badge.svg)](.github/workflows/ci.yml)
[![release status](https://github.com/matteopolak/basket/actions/workflows/release.yml/badge.svg)](.github/workflows/release.yml)
[![license](https://img.shields.io/github/license/matteopolak/basket.svg)](LICENSE)

A simple HTTP/1.1 client and server.

## Features

- Arbitrary headers
- JSON serialization/deserialization with [serde_json](https://github.com/serde-rs/json)
- XML serialization/deserialization with [quick-xml](https://github.com/tafia/quick-xml)
- `GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `OPTIONS` methods

## Examples

View all examples in [examples](examples) directory, and run them with `cargo run --package example-<name>`.
