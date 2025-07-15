use core::alloc::Layout;
use core::ptr::NonNull;

use crate::collections::DoublyLinkedList;
use crate::memory::frame::{BASE_SIZE, Frame, State};
use crate::memory::free_lists::FreeLists;
use crate::memory::{HartCache, PhysicalAddress, PhysicalMemoryMap};

const HART_CACHE_FRAMES: usize = 16;
const MAX_HARTS: usize = 12; // TODO: make dynamic

pub struct FrameAllocator {
    free_lists: FreeLists, // TODO: move lock here
    hart_caches: [HartCache<Frame, HART_CACHE_FRAMES>; MAX_HARTS],

    orders: u8,
    memory_map: *const PhysicalMemoryMap,
}

impl FrameAllocator {
    /// # Safety
    ///
    /// `pmem_map` must point to a valid, initialized `PhysicalMemoryMap` with
    /// page-aligned, non-overlapping regions for frame metadata and allocator data.
    ///
    /// These regions must be exclusively owned by the allocator and sized correctly.
    ///
    /// A raw pointer is used for performance and FFI-compatibility; no aliasing or concurrent access is allowed.
    pub unsafe fn init(pmem_map: *const PhysicalMemoryMap) -> Self {
        let memory_map = unsafe { &*pmem_map };
        // create frame metadata slice in the frame pool region
        let frame_slice = unsafe {
            core::slice::from_raw_parts_mut(
                memory_map.frame_pool.start().as_mut_ptr::<Frame>(),
                memory_map.num_frames(),
            )
        };

        assert_eq!(
            frame_slice.len(),
            memory_map.num_frames(),
            "Frame slice length doesn't match number of frames"
        );

        frame_slice.iter_mut().for_each(|frame| {
            *frame = Frame::new();
        });

        let orders = (memory_map.num_frames().ilog2() + 1) as u8;

        // create free intrusive list for each order in the frame allocator metadata region
        let free_lists = unsafe {
            core::slice::from_raw_parts_mut(
                memory_map
                    .frame_allocator_metadata
                    .start()
                    .as_mut_ptr::<DoublyLinkedList<Frame>>(),
                orders as usize,
            )
        };

        assert_eq!(
            free_lists.len(),
            orders as usize,
            "Free list count doesn't match orders"
        );

        free_lists.iter_mut().for_each(|list| {
            *list = DoublyLinkedList::new();
        });

        let mut free_lists = FreeLists::new(free_lists);

        let mut current_free_address = memory_map.free_memory.start();
        let mut frames_left = memory_map.free_memory.size() / BASE_SIZE;

        // greedy algorithm to distribute free memory blocks into free lists
        // starting from the highest order memory block available
        while frames_left > 0 {
            let largest_block_order = frames_left.ilog2();
            let largest_block_frames = 1 << largest_block_order;
            let largest_block_bytes = largest_block_frames * BASE_SIZE;

            let head_frame_idx = (current_free_address - memory_map.ram.start()) / BASE_SIZE;
            let head_frame = &mut frame_slice[head_frame_idx];

            head_frame.set_order(largest_block_order as u8);

            // set the frame with correspondng order as a head of the ordered free list
            free_lists.push_frame(NonNull::from(head_frame));

            frames_left -= largest_block_frames;
            current_free_address += largest_block_bytes;
        }

        assert_eq!(frames_left, 0, "Not all frames were initialized");
        assert_eq!(
            current_free_address,
            memory_map.free_memory.end(),
            "Uninitialized free memory detected"
        );

        // TODO: check initialization
        let hart_caches = core::array::from_fn(|_| HartCache::new());

        FrameAllocator {
            free_lists,
            hart_caches,
            orders,
            memory_map: pmem_map,
        }
    }

    pub fn orders(&self) -> u8 {
        self.orders
    }

    pub fn bitmap(&self) -> u64 {
        self.free_lists.bitmap_bits()
    }

    fn memory_map(&self) -> &PhysicalMemoryMap {
        unsafe { &*self.memory_map }
    }

    pub fn order_from_size(&self, size: usize) -> u8 {
        if size == 0 {
            return 0;
        }
        let frames = size.div_ceil(BASE_SIZE); // round up
        frames.next_power_of_two().ilog2() as u8
    }

    // TODO: cosider result return type with error types later
    pub fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        // TODO: decide if I want to allocate aligned-up size in that case
        if layout.align() > BASE_SIZE {
            return None;
        }

        let size = layout.size();

        if size == 0 {
            return Some(NonNull::dangling());
        }

        assert!(
            size < self.memory_map().free_memory.size(),
            "Requested size exceeds available memory"
        );

        let order = self.order_from_size(size);

        if order == 0 {
            match self.get_from_cache() {
                Some(head_frame) => return self.finalize_frame_allocation(head_frame),
                None =>
                // TODO: handle oom properly
                {
                    panic!(
                        "Out Of Memory: no free blocks available for order {}",
                        order
                    )
                }
            }
        }

        match self.prepare_block(order) {
            Some(head_frame) => self.finalize_frame_allocation(head_frame),
            None =>
            // TODO: handle oom properly
            {
                panic!(
                    "Out Of Memory: no free blocks available for order {}",
                    order
                )
            }
        }
    }

    fn finalize_frame_allocation(&self, mut frame_ptr: NonNull<Frame>) -> Option<NonNull<u8>> {
        let frame = unsafe { frame_ptr.as_mut() };
        frame.set_state(State::Allocated);
        let frame_addr = self.memory_map().frame_ref_to_address(frame);

        NonNull::new(frame_addr.as_mut_ptr::<u8>())
    }

    fn get_from_cache(&mut self) -> Option<NonNull<Frame>> {
        // Get the current hart's ID. This is architecture-specific.
        // You'll need to implement this for your architecture (e.g., by reading `mhartid` in RISC-V).
        // TODO:
        let hart_id = 1;

        if !self.hart_caches[hart_id].is_empty() {
            return self.hart_caches[hart_id].pop();
        }

        // SLOW PATH (REFILL): The cache is empty.
        // We need to grab a new batch from the global allocator.
        for _ in 0..self.hart_caches[hart_id].target_size() {
            if let Some(frame_ptr) = self.prepare_block(0) {
                self.hart_caches[hart_id].push(frame_ptr);
            } else {
                // Global allocator is out of order-0 frames.
                break;
            }
        }

        self.hart_caches[hart_id].pop()
    }

    fn prepare_block(&mut self, requested_order: u8) -> Option<NonNull<Frame>> {
        let found_order = self.free_lists.find_first_free_from(requested_order)?;

        let mut block_to_split = self.free_lists.pop_frame(found_order)?;

        // split the block down until it fits the requested order
        for current_order in (requested_order..found_order).rev() {
            let block_addr = self
                .memory_map()
                .frame_ref_to_address(unsafe { block_to_split.as_ref() });

            let buddy_offset = (1 << current_order) * BASE_SIZE;
            let buddy_addr = block_addr + buddy_offset;
            let mut buddy_frame_ptr = self.memory_map().address_to_frame_ptr(buddy_addr);
            let buddy_frame_ref = unsafe { buddy_frame_ptr.as_mut() };

            // downgrade blocks order, i.e `split`
            unsafe { block_to_split.as_mut().set_order(current_order) };
            buddy_frame_ref.set_order(current_order);

            self.free_lists.push_frame(NonNull::from(buddy_frame_ref));
        }

        Some(block_to_split)
    }

    pub fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        if layout.size() == 0 {
            return; // ZST dropped
        }

        let mut current_addr = PhysicalAddress::from(ptr.as_ptr() as usize);

        assert!(
            self.memory_map().ram.contains(current_addr),
            "Attempted to deallocate a pointer outside managed memory"
        );

        let mut current_frame_ptr = self.memory_map().address_to_frame_ptr(current_addr);
        let mut current_frame_ref = unsafe { current_frame_ptr.as_mut() };

        debug_assert!(
            !current_frame_ref.is_free(),
            "Double free detected at address {:#x}",
            current_addr.as_usize()
        );

        current_frame_ref.set_state(State::Free);

        let mut current_order = current_frame_ref.order();

        while current_order < self.orders - 1 {
            // calculate buddy address
            let buddy_offset = (1 << current_order) * BASE_SIZE;
            let buddy_addr = current_addr ^ buddy_offset;

            let mut buddy_frame_ptr = self.memory_map().address_to_frame_ptr(buddy_addr);
            let buddy_frame_ref = unsafe { buddy_frame_ptr.as_mut() };

            if buddy_frame_ref.is_free() && buddy_frame_ref.order() == current_order {
                // pass a copyable raw pointer to avoid moving the original reference
                self.free_lists.remove_frame(buddy_frame_ptr);

                // if the buddy has a lower address, it becomes the new block header
                if buddy_addr < current_addr {
                    current_addr = buddy_addr;
                    current_frame_ref = buddy_frame_ref;
                    current_frame_ptr = buddy_frame_ptr;
                }

                // increase the order for the new block
                current_order += 1;
                current_frame_ref.set_order(current_order);
            } else {
                // buddy is not free or is a different size, stop
                break;
            }
        }

        self.free_lists.push_frame(current_frame_ptr);
    }
}

unsafe impl Send for FrameAllocator {}
unsafe impl Sync for FrameAllocator {}
