#![no_std]
#![no_main]
// Modules
pub mod drivers;
pub mod globals;
pub mod macros;
pub mod sync;
pub mod trap;

// ---
use core::sync::atomic::AtomicBool;
use core::{fmt::Write, panic::PanicInfo};

use fdt::Fdt;

core::arch::global_asm!(include_str!("asm/boot.S"));
core::arch::global_asm!(include_str!("asm/trap.S"));

static IS_PANICKING: AtomicBool = AtomicBool::new(false);

#[panic_handler]
fn _panic(info: &PanicInfo) -> ! {
    // TODO: interrupt other harts here

    let mut stolen_uart = globals::UART_INSTANCE.steal().unwrap_or_else(|| halt());

    // TODO: write a crash log in a file or buffer

    if IS_PANICKING.swap(true, core::sync::atomic::Ordering::Relaxed) {
        stolen_uart
            .write_str("KERNEL PANIC: circular panic detected\n")
            .unwrap(); // unwrap is fine since write_str have no error case
    } else {
        stolen_uart
            .write_fmt(format_args!("KERNEL PANIC: {info}\n"))
            .map_err(|_| stolen_uart.write_str("KERNEL PANIC: Failed to write panic info"))
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
pub extern "C" fn kmain(hartid: usize, dtb_ptr: usize) -> ! {
    // Default UART base address, can be overridden by FDT
    let fdt = unsafe { Fdt::from_ptr(dtb_ptr as *const u8).unwrap() };

    drivers::probe_and_init_devices(&fdt);

    print_welcome_screen();
    panic!("This is a panic test on hart {}", hartid);
}

pub fn print_welcome_screen() {
    println!(
        r#"

██╗    ██╗ ███████╗ ██╗      ██████╗   ██████╗  ███╗   ███╗ ███████╗
██║    ██║ ██╔════╝ ██║     ██╔════╝  ██╔═══██╗ ████╗ ████║ ██╔════╝
██║ █╗ ██║ █████╗   ██║     ██║       ██║   ██║ ██╔████╔██║ █████╗
██║███╗██║ ██╔══╝   ██║     ██║       ██║   ██║ ██║╚██╔╝██║ ██╔══╝
╚███╔███╔╝ ███████╗ ███████╗╚██████╗  ╚██████╔╝ ██║ ╚═╝ ██║ ███████╗
 ╚══╝╚══╝  ╚══════╝ ╚══════╝ ╚═════╝   ╚═════╝  ╚═╝     ╚═╝ ╚══════╝

"#
    );
}
