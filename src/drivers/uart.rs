use super::{Device, Driver};
use crate::globals::UART_INSTANCE;
use crate::println;

use core::fmt;
use core::ptr::{read_volatile, write_volatile};
use embedded_io::{Error, ErrorKind, ErrorType, Write};
use fdt::node::FdtNode;

const LSR_OFFSET: usize = 5;
const LSR_TX_EMPTY: u8 = 1 << 5;

pub struct UartDriver;

impl Driver for UartDriver {
    type Device = Uart;

    fn init_global(&self, device: Self::Device) {
        let addr = device.base_address;
        let mut guard = UART_INSTANCE.lock();
        *guard = Some(device);
        drop(guard);
        println!("UART ns16550a initialized with base address: {:#x}", addr);
    }

    fn compatibility(&self) -> &'static [&'static str] {
        &["ns16550a", "riscv,ns16550a"]
    }

    fn probe(&self, node: &FdtNode) -> Option<Self::Device> {
        if !self.is_compatible(node) {
            return None;
        }

        let base_addr = node.reg()?.next()?.starting_address;
        let uart = Uart::new(base_addr as usize);

        Some(uart)
    }
}

pub struct Uart {
    pub base_address: usize,
}

impl Device for Uart {}

impl Uart {
    pub fn new(base_address: usize) -> Self {
        Self { base_address }
    }

    pub fn send_byte_blocking(&mut self, byte: u8) {
        let base_ptr = self.base_address as *mut u8;
        unsafe {
            // wait untill transmit holding register is empty (5th bit of LSR is set)
            loop {
                let lsr = read_volatile(base_ptr.add(LSR_OFFSET));
                if (lsr & LSR_TX_EMPTY) != 0 {
                    break;
                }
            }
            write_volatile(base_ptr, byte);
        }
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

// HAL Write trait, similar to io::Write
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
