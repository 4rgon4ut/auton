pub mod address;
pub mod frame;
pub mod frame_allocator;
pub mod free_lists;
pub mod hart_cache;
pub mod pmem_map;
pub mod slub;

pub use address::PhysicalAddress;
pub use frame_allocator::FrameAllocator;
pub use hart_cache::HartCache;
pub use pmem_map::PhysicalMemoryMap;
pub use slub::{KernelAllocator, SlubAllocator};

use crate::sync::OnceLock;
use fdt::standard_nodes::Memory;

// SAFETY: PhysicalMemoryMap is immutable
pub static PMEM_MAP: OnceLock<PhysicalMemoryMap> = OnceLock::new();
pub fn pmem_map() -> &'static PhysicalMemoryMap {
    PMEM_MAP
        .get()
        .expect("FATAL: PMEM_MAP accessed before initialization")
}

pub static FRAME_ALLOCATOR: OnceLock<FrameAllocator> = OnceLock::new();
pub fn frame_allocator() -> &'static FrameAllocator {
    FRAME_ALLOCATOR
        .get()
        .expect("FATAL: Frame allocator accessed before initialization")
}

#[global_allocator]
static KERNEL_ALLOCATOR: KernelAllocator = KernelAllocator::new();

// FIXME:
// #[alloc_error_handler]
// fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
//     panic!("Kernel allocation error: {:?}", layout);
// }

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

    match FRAME_ALLOCATOR.set(frame_allocator) {
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

    // TODO: check kernel allocator initialization
}
