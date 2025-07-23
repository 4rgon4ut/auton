use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::ptr::NonNull;

use crate::collections::DoublyLinkedList;
use crate::cpu::current_hart_id;
use crate::memory::frame::{BASE_SIZE, Frame, State};
use crate::memory::free_lists::FreeLists;
use crate::memory::hart_cache::{MAX_HARTS, Quartering};
use crate::memory::{HartCache, PhysicalAddress, PhysicalMemoryMap};
use crate::sync::Spinlock;

const DEFAULT_CACHE_SIZE: usize = 16;

pub struct FrameAllocator {
    free_lists: Spinlock<FreeLists>,
    hart_caches: [UnsafeCell<HartCache<Frame, Quartering>>; MAX_HARTS], // TODO: make dynamic based on number of harts

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
        let hart_caches = core::array::from_fn(|_| {
            UnsafeCell::new(HartCache::new(DEFAULT_CACHE_SIZE, Quartering))
        });

        FrameAllocator {
            free_lists: Spinlock::new(free_lists),
            hart_caches,
            orders,
            memory_map: pmem_map,
        }
    }

    pub fn orders(&self) -> u8 {
        self.orders
    }

    pub fn bitmap(&self) -> u64 {
        self.free_lists.lock().bitmap_bits()
    }

    #[inline]
    #[allow(clippy::mut_from_ref)]
    fn hart_cache(&self, hart_id: usize) -> &mut HartCache<Frame, Quartering> {
        unsafe { &mut *self.hart_caches[hart_id].get() }
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
    pub fn alloc(&self, layout: Layout) -> Option<NonNull<u8>> {
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

    pub fn alloc_slab(&self) -> Option<NonNull<Frame>> {
        self.get_from_cache()
    }

    fn finalize_frame_allocation(&self, mut frame_ptr: NonNull<Frame>) -> Option<NonNull<u8>> {
        let frame = unsafe { frame_ptr.as_mut() };
        frame.set_state(State::Allocated);
        let frame_addr = self.memory_map().frame_ref_to_address(frame);

        NonNull::new(frame_addr.as_mut_ptr::<u8>())
    }

    fn get_from_cache(&self) -> Option<NonNull<Frame>> {
        let hart_id = current_hart_id();
        let cache = self.hart_cache(hart_id);

        if !cache.is_empty() {
            return cache.pop();
        }

        // refill
        for _ in 0..cache.refill_amount() {
            if let Some(frame_ptr) = self.prepare_block(0) {
                cache.push(frame_ptr);
            } else {
                // global allocator is out of order-0 frames
                break;
            }
        }

        cache.pop()
    }

    fn prepare_block(&self, requested_order: u8) -> Option<NonNull<Frame>> {
        let mut free_lists = self.free_lists.lock();

        let found_order = free_lists.find_first_free_from(requested_order)?;

        let mut block_to_split = free_lists.pop_frame(found_order)?;

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

            free_lists.push_frame(NonNull::from(buddy_frame_ref));
        }

        Some(block_to_split)
    }

    pub fn dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
        if layout.size() == 0 {
            return; // ZST dropped
        }

        let current_addr = PhysicalAddress::from(ptr.as_ptr() as usize);

        assert!(
            self.memory_map().ram.contains(current_addr),
            "Attempted to deallocate a pointer outside managed memory"
        );

        let mut current_frame_ptr = self.memory_map().address_to_frame_ptr(current_addr);
        let current_frame_ref = unsafe { current_frame_ptr.as_mut() };

        debug_assert!(
            !current_frame_ref.is_free(),
            "Double free detected at address {:#x}",
            current_addr.as_usize()
        );

        current_frame_ref.set_state(State::Free);

        let order = current_frame_ref.order();

        if order > 0 {
            self.free_to_global(current_frame_ptr);
            return;
        }

        let hart_id = current_hart_id();
        let cache = self.hart_cache(hart_id);

        if !cache.is_full() {
            return cache.push(NonNull::from(current_frame_ref));
        }

        // trim full cache
        for _ in 0..cache.drain_amount() {
            let frame_to_free = cache.pop().unwrap();
            self.free_to_global(frame_to_free);
        }

        cache.push(current_frame_ptr);
    }

    fn free_to_global(&self, frame_ptr: NonNull<Frame>) {
        let mut current_frame_ptr = frame_ptr;
        let mut current_frame_ref = unsafe { current_frame_ptr.as_mut() };
        let mut current_addr = self.memory_map().frame_ref_to_address(current_frame_ref);
        let mut current_order = current_frame_ref.order();

        let mut free_lists = self.free_lists.lock();

        while current_order < self.orders - 1 {
            // calculate buddy address
            let buddy_offset = (1 << current_order) * BASE_SIZE;
            let buddy_addr = current_addr ^ buddy_offset;

            let mut buddy_frame_ptr = self.memory_map().address_to_frame_ptr(buddy_addr);
            let buddy_frame_ref = unsafe { buddy_frame_ptr.as_mut() };

            if buddy_frame_ref.is_free() && buddy_frame_ref.order() == current_order {
                // pass a copyable raw pointer to avoid moving the original reference
                free_lists.remove_frame(buddy_frame_ptr);

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

        free_lists.push_frame(current_frame_ptr);
    }
}

unsafe impl Send for FrameAllocator {}
unsafe impl Sync for FrameAllocator {}
