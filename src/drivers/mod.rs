pub mod clint;
pub mod uart;

pub use uart::UartDriver;

use fdt::node::FdtNode;

pub trait Driver {
    type Device: Device;

    fn init_global(&self, device: Self::Device);

    fn compatible(&self) -> &'static [&'static str];

    fn probe(&self, node: &FdtNode) -> Option<Self::Device>;
}

pub trait Device {}

pub fn probe_and_init_devices(fdt: &fdt::Fdt) {
    let drivers = [
        &UartDriver,
        // TODO:
    ];

    for node in fdt.all_nodes() {
        for driver in drivers.iter() {
            if let Some(device) = driver.probe(&node) {
                driver.init_global(device);
            }
        }
    }
}
