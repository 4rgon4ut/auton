use super::{Device, Driver};
use crate::devices::CLINT_INSTANCE;
use crate::sync::Spinlock;
use core::ptr::{read_volatile, write_volatile};

pub const MTIMECMP_OFFSET: usize = 0x4000;
pub const MTIME_OFFSET: usize = 0xBFF8;

pub const MSIP_HART_STRIDE: usize = 4;
pub const MTIMECMP_HART_STRIDE: usize = 8;

pub struct Clint {
    base_address: usize,
}

impl Clint {
    pub fn new(base_address: usize) -> Self {
        Self { base_address }
    }

    pub fn mtime(&self) -> u64 {
        let mtime_ptr = (self.base_address + MTIME_OFFSET) as *const u64; // MTIME is 64-bit
        unsafe { read_volatile(mtime_ptr) }
    }

    pub fn trigger_software_interrupt(&self, hart_id: usize) {
        self.write_msip(hart_id, 1);
    }

    pub fn clear_software_interrupt(&self, hart_id: usize) {
        self.write_msip(hart_id, 0);
    }

    pub fn schedule_timer_interrupt(&self, hart_id: usize, time: u64) {
        let mtimecmp_ptr =
            (self.base_address + MTIMECMP_OFFSET + MTIMECMP_HART_STRIDE * hart_id) as *mut u64; // MTIMECMP is 64-bit
        unsafe {
            write_volatile(mtimecmp_ptr, time);
        }
    }

    fn write_msip(&self, hart_id: usize, value: u32) {
        let msip_ptr = (self.base_address + MSIP_HART_STRIDE * hart_id) as *mut u32; // MSIP is 32-bit
        unsafe {
            write_volatile(msip_ptr, value);
        }
    }
}

impl Device for Clint {}

pub struct ClintDriver;

impl Driver for ClintDriver {
    type Device = Clint;

    fn init_global(&self, device: Self::Device) {
        let addr = device.base_address;

        CLINT_INSTANCE.get_or_init(|| Spinlock::new(device));

        let driver_type = self.compatibility()[0];
        println!(
            "[ OK ] CLINT ({}): successfully initialized at {:#x}",
            driver_type, addr
        );
    }

    fn compatibility(&self) -> &'static [&'static str] {
        &["riscv,clint0"]
    }

    fn probe(&self, node: &fdt::node::FdtNode) -> Option<Self::Device> {
        if !self.is_compatible(node) {
            return None;
        }

        let base_addr = node.reg()?.next()?.starting_address;
        let clint = Clint::new(base_addr as usize);

        Some(clint)
    }
}
