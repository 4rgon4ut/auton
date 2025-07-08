use super::address::PhysicalAddress;
use super::frame::{BASE_SIZE, Frame};
use crate::collections::IntrusiveList;

#[derive(Debug)]
pub struct MemoryRegion {
    start: PhysicalAddress,
    size: usize,
}

impl MemoryRegion {
    pub const fn new(start: PhysicalAddress, size: usize) -> Self {
        Self { start, size }
    }

    pub const fn start(&self) -> PhysicalAddress {
        self.start
    }

    pub const fn size(&self) -> usize {
        self.size
    }

    pub fn end(&self) -> PhysicalAddress {
        self.start + self.size
    }

    pub fn contains(&self, address: PhysicalAddress) -> bool {
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
    pub fn calculate(ram_start: PhysicalAddress, ram_size: usize) -> Self {
        // These symbols are defined by the linker script.
        unsafe extern "C" {
            static _kernel_start: [u8; 0];
            static _kernel_end: [u8; 0];
        }

        let kernel_start = unsafe { _kernel_start.as_ptr() as usize };
        let kernel_end = unsafe { _kernel_end.as_ptr() as usize };

        assert_eq!(ram_size % BASE_SIZE, 0, "RAM size is not page-aligned");
        assert!(
            kernel_start >= ram_start.as_usize() && kernel_end <= ram_start.as_usize() + ram_size,
            "Kernel is not loaded within the provided RAM region"
        );

        // Kernel Region
        let kernel_size = align_up(kernel_end - kernel_start, BASE_SIZE);
        let kernel_region = MemoryRegion::new(kernel_start.into(), kernel_size);

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
        assert_eq!(
            free_memory_start.as_usize() % BASE_SIZE,
            0,
            "Free memory region is not page-aligned"
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

    pub fn num_frames(&self) -> usize {
        self.ram.size() / BASE_SIZE
    }

    /// Returns corresponding frame pool index for a given physical address
    pub fn frame_idx_from_address(&self, address: PhysicalAddress) -> usize {
        assert!(self.ram.contains(address), "Address is out of bounds");

        address.offset_from(self.ram.start()) / BASE_SIZE
    }

    /// Converts a physical address to a mutable reference to the corresponding `Frame`
    /// metadata in the frame pool.
    pub fn address_to_frame_ref(&mut self, address: PhysicalAddress) -> &mut Frame {
        let frame_pool_ptr = self.frame_pool.start().as_mut_ptr::<Frame>();
        unsafe { &mut *frame_pool_ptr.add(self.frame_idx_from_address(address)) }
    }

    /// Converts a `Frame` metadata reference to the corresponding memory region start address
    pub fn frame_ref_to_address(&self, frame: &Frame) -> PhysicalAddress {
        let frame_addr = PhysicalAddress::new(frame as *const Frame as usize);
        let frame_idx =
            frame_addr.offset_from(self.frame_pool.start()) / core::mem::size_of::<Frame>();

        self.ram.start() + frame_idx * BASE_SIZE
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
