#![no_std]
#![no_main]

pub mod uart;

use core::panic::PanicInfo;

core::arch::global_asm!(include_str!("asm/boot.S"));

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain(hartid: usize, _dtb_ptr: usize) -> ! {
    loop {}
}
