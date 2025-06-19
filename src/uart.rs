use core::fmt;
use core::ptr::{read_volatile, write_volatile};
use embedded_io::{Error, ErrorKind, ErrorType, Write};

const BASE: *mut u8 = 0x1000_0000 as *mut u8;
const LSR_OFFSET: usize = 5;
const LSR_TX_EMPTY: u8 = 1 << 5;

pub static mut UART_INSTANCE: Uart = Uart::new();

pub struct Uart;

impl Uart {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn send_byte_blocking(&mut self, byte: u8) {
        unsafe {
            // wait untill transmit holding register is empty (5th bit of LSR is set)
            loop {
                let lsr = read_volatile(BASE.add(LSR_OFFSET));
                if (lsr & LSR_TX_EMPTY) != 0 {
                    break;
                }
            }
            write_volatile(BASE, byte);
        }
    }
}

impl Default for Uart {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct UartError;

impl Error for UartError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

impl ErrorType for Uart {
    type Error = UartError;
}

// HAL Write trait
impl Write for Uart {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        for &byte in buf {
            self.send_byte_blocking(byte);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        // no-op, QEMU virt UART does not require flushing
        Ok(())
    }
}

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}
