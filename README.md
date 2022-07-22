# rusty-bee

An attempt to make an open source ZigBee stack in embedded Rust.

This is mostly for my own purposes specifically for NRF52 chips but let's see
where it goes :)

## Installation

1. `rustup target add thumbv7em-none-eabihf`

2. `cargo build --target thumbv7em-none-eabihf`

## Testing

1. `cargo test`

## Directory Structure

* `rusty-bee` - The core, platform independent library

* `rusty-bee-nrf52840` - Implementation of the library for the NRF52840 SoC,
  written as a C library for now to work with the adafruit Android toolchain.
