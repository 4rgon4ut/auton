use crate::{
    devices::{_UART_PANIC_ADDRESS, UART_INSTANCE, uart},
    drivers::uart::Uart,
};
use core::fmt::{self, Write};

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let mut guard = uart();

    guard
        .write_fmt(args)
        .map_err(|e| {
            drop(guard);
            panic!("UART write error: {}", e);
        })
        .ok();
}

#[doc(hidden)]
pub fn _panic_print(args: fmt::Arguments) {
    // Try to use the fully initialized, primary UART driver.
    // This is the best-case scenario. It will succeed if the driver
    // is initialized and not currently locked.
    if let Some(mut guard) = UART_INSTANCE.get().and_then(|lock| lock.try_lock()) {
        guard.write_fmt(args).ok();
        return;
    }

    // Fallback: The primary driver is unavailable. Try the panic address.
    // We can only `get()` the address. If it hasn't been set yet,
    // it's too late to initialize it now, so we can't print.
    if let Some(panic_addr) = _UART_PANIC_ADDRESS.get() {
        let mut stolen_uart = Uart::new(*panic_addr);
        stolen_uart.write_fmt(args).ok();
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::printing::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
