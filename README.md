![CI workflow](https://github.com/swaagie/envoy-rbi-filter/actions/workflows/ci.yml/badge.svg)

# Envoy Response Body Injection filter

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
cargo build --target wasm32-wasi --release
docker compose up --build --file ./example/docker-compose.yaml
```

Send the local running cluster traffic:

```sh
curl -vvv http://localhost:10000/

# *   Trying ::1...
# * TCP_NODELAY set
# ...
# * Connection #0 to host localhost left intact
# <html><body><h1>Hello WASM</h1></body></html>
# * Closing connection 0
```