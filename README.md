![CI workflow](https://github.com/swaagie/envoy-rbi-filter/actions/workflows/ci.yml/badge.svg)

# Envoy Response Body Injection filter

_Note: this Envoy filter is a prove of concept and not production ready_

Envoy filter written in Rust to provide Reponse Body Injection (RBI). This requires [Envoy's WASM filters](https://www.envoyproxy.io/docs/envoy/latest/configuration/http/http_filters/wasm_filter.html?highlight=wasm)

## Build

```sh
rustup target add wasm32-wasi
cargo build --target wasm32-wasi
```

Release build

```sh
rustup target add wasm32-wasi
cargo build --target wasm32-wasi --release
```

## Usage


## Example

To run the example local:

```sh
cd example
docker compose up --build
```

Send

## Features to add

- Insert before, only append is supported
- CSS selector node querying