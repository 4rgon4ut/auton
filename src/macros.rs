use crate::uart::UART_INSTANCE;
use core::fmt::{self, Write};

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    unsafe {
        let uart_ptr = core::ptr::addr_of_mut!(UART_INSTANCE);
        // TODO: add locking mechanism to prevent concurrent writes
        *uart_ptr.write_fmt(args).unwrap();
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::macros::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
