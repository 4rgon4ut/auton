use crate::drivers::{Clint, Uart};
use crate::sync::Spinlock;

pub static UART_INSTANCE: Spinlock<Option<Uart>> = Spinlock::new(None);
pub static CLINT_INSTANCE: Spinlock<Option<Clint>> = Spinlock::new(None);
