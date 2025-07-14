pub mod address;
pub mod frame;
pub mod frame_allocator;
pub mod free_lists;
pub mod pmem_map;
pub mod slub;

pub use address::PhysicalAddress;
pub use frame_allocator::FrameAllocator;
pub use pmem_map::PhysicalMemoryMap;

use crate::sync::{OnceLock, Spinlock};
use core::alloc::GlobalAlloc;
use fdt::standard_nodes::Memory;

struct KernelAllocator;

// #[global_allocator]
// pub static ALLOCATOR: KernelAllocator = KernelAllocator;

// SAFETY: PhysicalMemoryMap is immutable
pub static PMEM_MAP: OnceLock<PhysicalMemoryMap> = OnceLock::new();

pub static FRAME_ALLOCATOR: OnceLock<Spinlock<FrameAllocator>> = OnceLock::new();
unsafe impl Send for FrameAllocator {}

pub fn init(memory: Memory) {
    let main_region = memory
        .regions()
        .next()
        .expect("No memory regions defined in FDT");

    let ram_start = PhysicalAddress::new(main_region.starting_address as usize);
    let ram_size = main_region
        .size
        .expect("No size defined for the main memory region");

    let pmem_map = PhysicalMemoryMap::calculate(ram_start, ram_size);

    PMEM_MAP.set(pmem_map).expect("Failed to set PMEM_MAP");
    println!("{}", PMEM_MAP.get().unwrap());

    let frame_allocator = unsafe {
        FrameAllocator::init(PMEM_MAP.get().expect("PMEM_MAP not set") as *const PhysicalMemoryMap)
    };

    let orders = frame_allocator.orders();
    let bitmap = frame_allocator.bitmap();

    match FRAME_ALLOCATOR.set(Spinlock::new(frame_allocator)) {
        Ok(_) => {
            println!(
                "[ OK ] FrameAllocator successfully initialized (orders: {}, bitmap: {:b})",
                orders, bitmap
            );
        }
        Err(_) => {
            panic!("Failed to initialize frame allocator");
        }
    }
}

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        panic!("Not implemented")
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        panic!("Not implemented")
    }
}
