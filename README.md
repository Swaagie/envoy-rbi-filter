![CI workflow](https://github.com/swaagie/envoy-rbi-filter/actions/workflows/ci.yml/badge.svg)

# Envoy RBI filter

**This filter is not ready for use yet**

Envoy filter written in Rust to provide Reponse Body Injection (RBI).
This requires [Envoy's WASM filters](https://www.envoyproxy.io/docs/envoy/latest/configuration/http/http_filters/wasm_filter.html?highlight=wasm)

Tests to add:
- benchmarks
- fuzz tests

Features to add:
- Insert before, only append is supported
- CSS4 node queries
