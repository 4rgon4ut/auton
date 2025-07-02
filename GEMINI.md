
# rvos Project Context

## Project Overview
- **Name:** rvos
- **Description:** A hobby operating system kernel written in Rust for the RISC-V 64GC architecture. It is designed to run on the QEMU `virt` machine.
- **Language:** Rust (2024 Edition)
- **Architecture:** RISC-V (`riscv64gc-unknown-none-elf`)

## Building and Running
- The project is built and run using the standard `cargo run` command.
- The runner configuration in `.cargo/config.toml` specifies the QEMU command and flags for execution.

## Key Files and Directories
- **`src/kmain.rs`**: The main entry point of the kernel.
- **`src/asm/boot.S`**: The initial assembly boot code that sets up the stack and jumps to the Rust kernel.
- **`src/lds/virt.lds`**: The linker script that defines the memory layout for the kernel.
- **`.cargo/config.toml`**: Contains the build target, rustflags, and the QEMU runner configuration.
- **`Cargo.toml`**: Defines project metadata and dependencies, such as `embedded-io` and `fdt`.
- **`src/drivers/`**: Contains device drivers, such as for the UART and CLINT.
- **`src/trap/`**: Contains trap handling code.
- **`src/sync/`**: Contains synchronization primitives.
