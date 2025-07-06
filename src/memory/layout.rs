use super::frame;

#[derive(Debug)]
pub struct MemoryRegion {
    pub start: usize,
    pub size: usize,
}

impl MemoryRegion {
    pub fn new(start: usize, size: usize) -> Self {
        Self { start, size }
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
    /// The region occupied by the kernel's binary (.text, .rodata, .data, .bss).
    pub kernel: MemoryRegion,

    /// The total available physical RAM discovered from the hardware.
    pub ram: MemoryRegion,

    /// The region reserved within RAM to store the `Frame` metadata array.
    /// This array tracks the state of every frame in the system.
    pub frame_pool: MemoryRegion,

    /// The region reserved within RAM to store the allocator's internal data.
    pub frame_allocator_data: MemoryRegion,

    /// The start address of the first physical page that is available for
    /// general-purpose allocation by the frame allocator.
    pub free_memory_start: usize,
}

// TODO: define in linker
unsafe extern "C" {
    // These symbols are defined by the linker script.
    // They are just addresses, so we declare them as empty arrays
    // of type `u8` to get their address.
    static _kernel_start: [u8; 0];
    static _kernel_end: [u8; 0];
}

impl Layout {
    pub fn calculate(ram_start: usize, ram_size: usize) -> Self {
        let kernel_start = unsafe { _kernel_start.as_ptr() as usize };
        let kernel_end = unsafe { _kernel_end.as_ptr() as usize };

        Layout {
            kernel: MemoryRegion::new(kernel_start, kernel_end - kernel_start),
            ram: MemoryRegion::new(ram_start, ram_size),
            frame_pool: MemoryRegion::new(0, 0),
            frame_allocator_data: MemoryRegion::new(0, 0),
            free_memory_start: 0, // FIXME
        }
    }
}
