#![no_std]
#![no_main]
// Modules
#[macro_use]
pub mod printing;
pub mod collections;
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

    let memory_region = fdt
        .memory()
        .regions()
        .next()
        .expect("No memory regions defined in FDT");

    // 5. Extract the start address and size from the FDT region
    let ram_start = memory::PhysicalAddress::new(memory_region.starting_address as usize);
    let ram_size = memory_region.size.unwrap_or(0);

    let pmem_map = memory::PhysicalMemoryMap::calculate(ram_start, ram_size);

    println!("{}", pmem_map);

    // print_welcome_screen();

    panic!("Test panic on hart {}", hart_id);
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
