use super::frame::{BASE_SIZE, Frame};
use crate::collections::IntrusiveList;

#[derive(Debug)]
pub struct MemoryRegion {
    start: usize,
    size: usize,
}

impl MemoryRegion {
    pub fn new(start: usize, size: usize) -> Self {
        Self { start, size }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn end(&self) -> usize {
        self.start + self.size
    }

    pub fn contains(&self, address: usize) -> bool {
        address >= self.start && address < self.end()
    }
}

#[derive(Debug)]
pub struct Layout {
    /// The total available physical RAM discovered from the hardware.
    pub ram: MemoryRegion,

    /// The region occupied by the kernel's binary (.text, .rodata, .data, .bss).
    pub kernel: MemoryRegion,

    /// The region reserved within RAM to store the `Frame` metadata array.
    /// This array tracks the state of every frame in the system.
    pub frame_pool: MemoryRegion,

    /// The region reserved within RAM to store the allocator's internal data.
    pub frame_allocator_metadata: MemoryRegion,

    /// The start address of the first physical page that is available for
    /// general-purpose allocation by the frame allocator.
    pub free_memory: MemoryRegion,
}

impl Layout {
    pub fn calculate(ram_start: usize, ram_size: usize) -> Self {
        // These symbols are defined by the linker script.
        unsafe extern "C" {
            static _kernel_start: [u8; 0];
            static _kernel_end: [u8; 0];
        }

        let kernel_start = unsafe { _kernel_start.as_ptr() as usize };
        let kernel_end = unsafe { _kernel_end.as_ptr() as usize };

        assert_eq!(ram_size % BASE_SIZE, 0, "RAM size is not page-aligned");
        assert!(
            kernel_start >= ram_start && kernel_end <= ram_start + ram_size,
            "Kernel is not loaded within the provided RAM region"
        );

        // Kernel Region
        let kernel_size = align_up(kernel_end - kernel_start, BASE_SIZE);
        let kernel_region = MemoryRegion::new(kernel_start, kernel_size);

        // Frame Pool Region
        let num_frames = ram_size / BASE_SIZE;
        let frame_pool_size = align_up(num_frames * size_of::<Frame>(), BASE_SIZE);
        let frame_pool_region = MemoryRegion::new(kernel_region.end(), frame_pool_size);

        // Allocator Data Region
        let allocator_num_orders = (num_frames.ilog2() + 1) as usize;
        let allocator_metadata_size = align_up(
            allocator_num_orders * size_of::<IntrusiveList<Frame>>(),
            BASE_SIZE,
        );
        let allocator_metadata_region =
            MemoryRegion::new(frame_pool_region.end(), allocator_metadata_size);

        // Free Memory Region
        let free_memory_start = allocator_metadata_region.end();

        assert!(
            free_memory_start <= ram_start + ram_size,
            "Not enough memory for kernel and metadata"
        );

        let free_memory_size = ram_start + ram_size - free_memory_start;
        let free_memory_region = MemoryRegion::new(free_memory_start, free_memory_size);

        Layout {
            ram: MemoryRegion::new(ram_start, ram_size),
            kernel: kernel_region,
            frame_pool: frame_pool_region,
            frame_allocator_metadata: allocator_metadata_region,
            free_memory: free_memory_region,
        }
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
