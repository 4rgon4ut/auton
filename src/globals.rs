use crate::drivers::uart::Uart;
use crate::sync::Spinlock;

pub static UART_INSTANCE: Spinlock<Option<Uart>> = Spinlock::new(None);
