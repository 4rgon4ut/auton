# Design

This document outlines the architecture and design choices of the rvos kernel.

## Boot Process

The boot process begins with the assembly code in `src/asm/boot.S`. This code is responsible for:

1.  **Hart Jail:** All harts except for hart 0 are put into a holding pattern (`hart_jail`).
2.  **Pointer Setup:** The stack pointer and global pointer are initialized.
3.  **BSS Clearing:** The BSS section is cleared to zero.
4.  **PMP Configuration:** Physical Memory Protection (PMP) is configured to allow access to all of memory.
5.  **Trap Delegation:** All traps are delegated to S-mode.
6.  **S-Mode Transition:** The processor is transitioned from M-mode to S-mode.
7.  **Kernel Jump:** The code then jumps to the `kmain` function in `src/kmain.rs`.

## Memory Management

The kernel uses a combination of a buddy allocator and a SLUB allocator for memory management.

*   **Buddy Allocator (`memory/frame.rs`):** This allocator is responsible for managing physical memory frames. It can allocate and deallocate blocks of memory of various sizes.
*   **SLUB Allocator (`memory/slub.rs`):** This allocator is built on top of the buddy allocator and is used for managing small, fixed-size objects. It is more efficient than the buddy allocator for this purpose.

## Drivers

Device drivers are located in the `drivers` directory. The kernel uses a trait-based driver model, where each driver implements the `Driver` trait. This allows for a generic and extensible driver framework.

The `probe_and_init_devices` function in `drivers/mod.rs` is responsible for probing and initializing all available devices. It iterates through the device tree and calls the `probe` function of each driver to check if the driver is compatible with the device.

## Trap Handling

Trap handling is implemented in the `trap` directory. The `trap_handler` function in `trap/handlers.rs` is the main entry point for all traps. It determines the cause of the trap and calls the appropriate handler.

The `TrapFrame` struct in `trap/traps.rs` contains the state of the processor at the time of the trap.

## Synchronization

The `sync` directory contains synchronization primitives used to protect shared data structures from race conditions.

*   **`Spinlock`:** A simple spinlock that provides mutual exclusion. It is used to protect data that is accessed by multiple harts.
*   **`OnceLock`:** A one-time initialization lock. It is used to ensure that a piece of data is initialized only once.

## Global Instances

The kernel uses global instances to provide access to core services, such as the UART driver. These instances are typically wrapped in a `OnceLock` to ensure that they are initialized only once.