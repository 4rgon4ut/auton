use crate::drivers::{Clint, Uart};
use crate::sync::{OnceLock, Spinlock, SpinlockGuard};

pub static _UART_PANIC_ADDRESS: OnceLock<usize> = OnceLock::new();
pub static UART_INSTANCE: OnceLock<Spinlock<Uart>> = OnceLock::new();

pub fn uart() -> SpinlockGuard<'static, Uart> {
    UART_INSTANCE
        .get()
        .expect("UART driver not initialized")
        .lock()
}

pub static CLINT_INSTANCE: OnceLock<Spinlock<Clint>> = OnceLock::new();

pub fn clint() -> SpinlockGuard<'static, Clint> {
    CLINT_INSTANCE
        .get()
        .expect("CLINT driver not initialized")
        .lock()
}
