#![no_std]
#![no_main]

pub mod uart;

use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};

const UART_BASE: *mut u8 = 0x1000_0000 as *mut u8;
const UART_THR_OFFSET: usize = 0;
const UART_LSR_OFFSET: usize = 5;
const UART_LSR_TX_EMPTY: u8 = 1 << 5;

core::arch::global_asm!(include_str!("asm/boot.S"));

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

fn put_char(c: char) {
    unsafe {
        loop {
            let lsr = read_volatile(UART_BASE.add(UART_LSR_OFFSET));
            if (lsr & UART_LSR_TX_EMPTY) != 0 {
                break;
            }
        }
        write_volatile(UART_BASE.add(UART_THR_OFFSET), c as u8);
    }
}

fn print(s: &str) {
    for c in s.chars() {
        put_char(c);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain(hartid: usize, _dtb_ptr: usize) -> ! {
    print("Welcome to rvos in S-Mode!\n");
    print("=========================\n");
    print("I am running on hart ");
    put_char((hartid as u8 + b'0') as char);
    print("\n");

    loop {}
}
