pub mod clint;
pub mod uart;

pub use clint::{Clint, ClintDriver};
pub use uart::{Uart, UartDriver};

use fdt::node::FdtNode;

pub trait Driver {
    type Device: Device;

    fn init_global(&self, device: Self::Device);

    fn compatibility(&self) -> &'static [&'static str];

    fn probe(&self, node: &FdtNode) -> Option<Self::Device>;

    fn is_compatible(&self, node: &FdtNode) -> bool {
        let compatibility_list = match node.compatible() {
            Some(list) => list,
            None => return false,
        };
        compatibility_list
            .all()
            .any(|c| self.compatibility().contains(&c))
    }
}

pub trait Device {}

macro_rules! probe_all_drivers {
    ($fdt_node:expr, $($driver:expr),+ $(,)?) => {
        // This code block will be expanded by the macro
        $(
            if let Some(device) = $driver.probe($fdt_node) {
                $driver.init_global(device);
            }
        )+
    };
}

pub fn probe_and_init_devices(fdt: &fdt::Fdt) {
    // TODO: make sure UART always initialized first
    for node in fdt.all_nodes() {
        probe_all_drivers!(&node, &UartDriver, &ClintDriver);
    }
}
