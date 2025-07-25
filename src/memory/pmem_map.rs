use crate::collections::DoublyLinkedList;
use crate::memory::address::PhysicalAddress;
use crate::memory::frame::{BASE_SIZE, Frame};

use core::fmt;
use core::ptr::NonNull;

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
pub struct PhysicalMemoryMap {
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

impl PhysicalMemoryMap {
    pub fn calculate(ram_start: PhysicalAddress, ram_size: usize) -> Self {
        let ram = MemoryRegion::new(ram_start, ram_size);

        assert_eq!(ram.size() % BASE_SIZE, 0, "RAM size is not page-aligned");

        let kernel_region = Self::init_kernel_region(&ram);

        let frame_pool_region = Self::init_frame_pool_region(&ram, kernel_region.end());

        let allocator_metadata_region =
            Self::init_allocator_metadata_region(&ram, frame_pool_region.end());

        let free_memory_region =
            Self::init_free_memory_region(&ram, allocator_metadata_region.end());

        PhysicalMemoryMap {
            ram,
            kernel: kernel_region,
            frame_pool: frame_pool_region,
            frame_allocator_metadata: allocator_metadata_region,
            free_memory: free_memory_region,
        }
    }

    // INITIALIZERS

    //
    fn init_kernel_region(ram: &MemoryRegion) -> MemoryRegion {
        // these symbols are defined by the linker script
        unsafe extern "C" {
            static _kernel_start: [u8; 0];
            static _kernel_end: [u8; 0];
        }

        let kernel_start = unsafe { _kernel_start.as_ptr() as usize };
        let kernel_end = unsafe { _kernel_end.as_ptr() as usize };

        assert!(
            ram.contains(kernel_start.into()),
            "Kernel start address is out of RAM bounds"
        );

        let kernel_size = align_up(kernel_end - kernel_start, BASE_SIZE);

        assert!(
            ram.contains((kernel_start + kernel_size).into()),
            "Kernel end address is out of RAM bounds"
        );

        MemoryRegion::new(kernel_start.into(), kernel_size)
    }

    fn init_frame_pool_region(
        ram: &MemoryRegion,
        kernel_region_end: PhysicalAddress,
    ) -> MemoryRegion {
        let num_frames = ram.size() / BASE_SIZE;
        let frame_pool_size = align_up(num_frames * size_of::<Frame>(), BASE_SIZE);

        assert!(
            ram.contains(kernel_region_end + frame_pool_size),
            "Frame Pool Region end address is out of RAM bounds"
        );

        MemoryRegion::new(kernel_region_end, frame_pool_size)
    }

    fn init_allocator_metadata_region(
        ram: &MemoryRegion,
        frame_pool_end: PhysicalAddress,
    ) -> MemoryRegion {
        let num_frames = ram.size() / BASE_SIZE;
        let allocator_num_orders = (num_frames.ilog2() + 1) as usize;
        let allocator_metadata_size = align_up(
            allocator_num_orders * size_of::<DoublyLinkedList<Frame>>(),
            BASE_SIZE,
        );

        assert!(
            ram.contains(frame_pool_end + allocator_metadata_size),
            "Frame Allocator Metadata Region end address is out of RAM bounds"
        );

        MemoryRegion::new(frame_pool_end, allocator_metadata_size)
    }

    fn init_free_memory_region(
        ram: &MemoryRegion,
        allocator_metadata_end: PhysicalAddress,
    ) -> MemoryRegion {
        let free_memory_start = allocator_metadata_end;

        assert_eq!(
            free_memory_start.as_usize() % BASE_SIZE,
            0,
            "Free memory region is not page-aligned"
        );

        let free_memory_size = ram.end() - free_memory_start;

        MemoryRegion::new(free_memory_start, free_memory_size)
    }

    pub fn num_frames(&self) -> usize {
        self.ram.size() / BASE_SIZE
    }

    /// Returns corresponding frame pool index for a given physical address
    pub fn frame_idx_from_address(&self, address: PhysicalAddress) -> usize {
        assert!(self.ram.contains(address), "Address is out of bounds");

        address.offset_from(self.ram.start()) / BASE_SIZE
    }

    /// Converts a physical address to a raw pointer to the corresponding `Frame`
    /// metadata in the frame pool.
    ///
    /// # SAFETY
    /// Pointer is guaranteed to be valid and properly aligned,
    /// since the index is bounds-checked in `frame_idx_from_address()`.
    pub fn address_to_frame_ptr(&self, address: PhysicalAddress) -> NonNull<Frame> {
        let frame_pool_ptr = self.frame_pool.start().as_mut_ptr::<Frame>();
        let frame_ptr = unsafe { frame_pool_ptr.add(self.frame_idx_from_address(address)) };

        unsafe { NonNull::new_unchecked(frame_ptr) }
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

impl fmt::Display for MemoryRegion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:>18} ──> {:>18} | {:>8} KiB",
            self.start(),
            self.end(),
            self.size() / 1024
        )
    }
}

impl fmt::Display for PhysicalMemoryMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let line = "═══════════════════════════════════════════════════════";

        writeln!(f)?;
        writeln!(f, "PHYSICAL MEMORY LAYOUT")?;
        writeln!(f, "{line}")?;

        let regions = [
            ("Kernel", &self.kernel),
            ("Frame Pool", &self.frame_pool),
            ("Allocator", &self.frame_allocator_metadata),
            ("Free RAM", &self.free_memory),
        ];

        for (name, region) in regions {
            writeln!(f, "{name:<12} | {region}")?;
        }
        writeln!(f, "{line}")?;

        writeln!(f, "Total RAM:    {} KiB", self.ram.size() / 1024)?;
        writeln!(f, "Total Frames: {}", self.num_frames())?;

        Ok(())
    }
}
