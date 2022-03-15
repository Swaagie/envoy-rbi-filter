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

Add the WASM filter configuration to `http_filters` as part of Envoy's HTTP Connection Management configuration.

```
- name: envoy.filters.http.wasm
	typed_config:
		"@type": type.googleapis.com/udpa.type.v1.TypedStruct
		type_url: type.googleapis.com/envoy.extensions.filters.http.wasm.v3.Wasm
		value:
			config:
				name: "rbi_filter"
				root_id: "rbi_filter_id"
				configuration:
					"@type": "type.googleapis.com/google.protobuf.StringValue"
					value: |
						{
							"hello": "<h1>Hello WASM</h1>"
						}
				vm_config:
					runtime: "envoy.wasm.runtime.v8"
					vm_id: "rbi_injection_vm_id"
					code:
						local:
							filename: "/etc/envoy/envoy_rbi_filter.wasm"
					configuration: {}
```

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

## Tests

End to end integration tests require the [example to run local](#example).

```sh
cargo test
```
