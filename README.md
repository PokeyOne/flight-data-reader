# Flight Data Reader

A reader for binary data in a custom binary format created for UVic Rocketry.
This repo contains three main components:
* **Rust library**: The actual implementation of conversion
* **CLI**: The command line interface for interacting with the rust library
* **WASM NPM package**: NPM WASM implementation for use in a ground station web library (by UVic Rocketry's Ground Support application).