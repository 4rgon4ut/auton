#![no_std]
#![no_main]

// Modules
#[macro_use]
pub mod printing;
pub mod collections;
pub mod cpu;
pub mod devices;
pub mod drivers;
pub mod memory;
pub mod sync;
pub mod trap;

// ---

use crate::printing::_panic_print;
use core::arch::global_asm;
use core::panic::PanicInfo;
use core::sync::atomic::AtomicBool;
use fdt::Fdt;

// boot code
global_asm!(include_str!("asm/boot.S"));
global_asm!(include_str!("asm/trap.S"));

static IS_PANICKING: AtomicBool = AtomicBool::new(false);

#[panic_handler]
fn _panic(info: &PanicInfo) -> ! {
    // TODO: interrupt other harts here
    // TODO: disable irqs for this hart
    // TODO: write a crash log in a file or buffer

    if IS_PANICKING.swap(true, core::sync::atomic::Ordering::Relaxed) {
        _panic_print(format_args!("KERNEL PANIC: circular panic detected\n"));
        halt();
    } else {
        _panic_print(format_args!("KERNEL PANIC: {info}\n"));
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
pub extern "C" fn kmain(hart_id: usize, dtb_ptr: usize) -> ! {
    // Default UART base address, can be overridden by FDT
    let fdt = unsafe { Fdt::from_ptr(dtb_ptr as *const u8).unwrap() };

    drivers::probe_and_init_devices(&fdt);

    memory::init(fdt.memory());

    print_welcome_screen();
    panic!("Test panic on hart {}", hart_id);
}

#[cfg(miri)]
#[unsafe(no_mangle)]
pub fn miri_start(_argc: isize, _argv: *const *const u8) -> isize {
    // Embed the project's virt.dtb so Fdt::from_ptr in kmain can parse it under Miri.
    let dtb = include_bytes!("../virt.dtb");
    kmain(0, dtb.as_ptr() as usize)
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
