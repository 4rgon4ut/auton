use crate::drivers::uart::UART_INSTANCE;
use core::fmt::{self, Write};

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let mut guard = UART_INSTANCE.lock();
    guard
        .write_fmt(args)
        .map_err(|e| {
            drop(guard);
            panic!("UART write error: {}", e);
        })
        .ok();
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
