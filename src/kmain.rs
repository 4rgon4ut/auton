#![no_std]
#![no_main]
// Modules
pub mod macros;
pub mod sync;
pub mod uart;

// ---
use core::sync::atomic::AtomicBool;
use core::{fmt::Write, panic::PanicInfo};

use crate::uart::{UART_INSTANCE, Uart};

core::arch::global_asm!(include_str!("asm/boot.S"));

static IS_PANICKING: AtomicBool = AtomicBool::new(false);

#[panic_handler]
fn _panic(info: &PanicInfo) -> ! {
    // TODO: interrupt other harts here

    let mut direct_uart = Uart::new();

    // TODO: write a crash log in a file or buffer

    if IS_PANICKING.swap(true, core::sync::atomic::Ordering::Relaxed) {
        direct_uart
            .write_str("KERNEL PANIC: circular panic detected\n")
            .unwrap(); // unwrap is fine since write_str have no error case
    } else {
        direct_uart
            .write_fmt(format_args!("KERNEL PANIC: {info}\n"))
            .map_err(|_| direct_uart.write_str("KERNEL PANIC: Failed to write panic info"))
            .ok();
    }

    halt();
}

fn halt() -> ! {
    unsafe {
        loop {
            core::arch::asm!("wfi");
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain(hartid: usize, _dtb_ptr: usize) -> ! {
    let _guard = UART_INSTANCE.lock();
    panic!("This is a panic test on hart {}", hartid);
}
