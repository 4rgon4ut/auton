# rvos

This is a pet project to write a small Kernel in Rust for the [RISC-V 64GC](https://riscv.org/) architecture, targeting the [QEMU](https://www.qemu.org/) virt machine.

## Dependencies

A minimal dependency approach is used in this project. The following dependencies are used:
*   [`fdt`](https://crates.io/crates/fdt): Used for parsing the device tree to avoid the redundant work of parsing it manually (boring).
*   [`embedded-io`](https://crates.io/crates/embedded-io): This crate provides Hardware Abstraction Layer (HAL) traits for I/O operations, which are used for convenience.

## Nightly Features

This project uses nightly Rust features. You will need to use a nightly toolchain to build and run it.

## Setup

1.  Install a nightly Rust toolchain. You can do this using `rustup`:
    ```sh
    rustup toolchain install nightly
    rustup default nightly
    ```
2.  Add the `riscv64gc-unknown-none-elf` target:
    ```sh
    rustup target add riscv64gc-unknown-none-elf
    ```
3.  Install QEMU. On macOS, you can use Homebrew:
    ```sh
    brew install qemu
    ```

## Running

To run the OS in QEMU, use the following command:

```sh
cargo run
```
