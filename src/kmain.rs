#![no_std]
#![no_main]
// Modules
pub mod macros;
pub mod sync;
pub mod uart;

// ---
use core::panic::PanicInfo;

core::arch::global_asm!(include_str!("asm/boot.S"));

#[panic_handler]
fn _panic(info: &PanicInfo) -> ! {
    println!("KERNEL PANIC: {}", info);
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
    println!("Hello, {}", "world!");
    panic!("This is a panic test on hart {}", hartid);
}
