use embedded_io::Write;

pub struct UART {
    base_address: *mut u8,
}

impl UART {
    pub fn new(addr: usize) -> Self {
        UART {
            base_address: addr as *mut u8,
        }
    }
}

impl Write for UART {}
