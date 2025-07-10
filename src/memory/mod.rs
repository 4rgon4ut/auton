pub mod address;
pub mod frame;
pub mod free_lists;
pub mod pmem_map;
pub mod slub;

pub use address::PhysicalAddress;
pub use frame::FrameAllocator;
pub use pmem_map::PhysicalMemoryMap;

use crate::sync::{OnceLock, Spinlock};
use core::alloc::GlobalAlloc;

struct KernelAllocator;

// FIXME
#[global_allocator]
pub static ALLOCATOR: KernelAllocator = KernelAllocator;

// pub static FRAME_ALLOCATOR: OnceLock<Spinlock<FrameAllocator>> = OnceLock::new();

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        panic!("Not implemented")
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        panic!("Not implemented")
    }
}
