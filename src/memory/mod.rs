use core::alloc::GlobalAlloc;

pub mod frame;
pub mod layout;
pub mod slub;

struct KernelAllocator;

// FIXME
#[global_allocator]
static ALLOCATOR: KernelAllocator = KernelAllocator;

// FIXME
unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        panic!("Not implemented")
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        panic!("Not implemented")
    }
}
