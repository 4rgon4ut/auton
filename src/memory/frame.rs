use crate::collections::{IntrusiveList, Linkable};
use crate::memory::Layout;
use core::ptr::NonNull;

#[derive(Debug, Clone, Copy)]
pub enum State {
    Free,
    Allocated,
}

#[derive(Debug)]
pub struct Frame {
    next: Option<NonNull<Frame>>,
    prev: Option<NonNull<Frame>>,

    order: u8,
    state: State,
}

impl Frame {
    pub const fn new() -> Self {
        Frame {
            next: None,
            prev: None,
            order: 0,
            state: State::Free,
        }
    }

    pub fn order(&self) -> u8 {
        self.order
    }

    pub fn set_order(&mut self, order: u8) {
        self.order = order;
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }

    pub fn is_free(&self) -> bool {
        matches!(self.state, State::Free)
    }

    pub fn size(&self) -> usize {
        (1 << self.order) * BASE_SIZE
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Linkable for Frame {
    fn next(&self) -> Option<NonNull<Self>> {
        self.next
    }

    fn prev(&self) -> Option<NonNull<Self>> {
        self.prev
    }

    fn set_next(&mut self, next: Option<NonNull<Self>>) {
        self.next = next;
    }

    fn set_prev(&mut self, prev: Option<NonNull<Self>>) {
        self.prev = prev;
    }
}

pub const BASE_SIZE: usize = 4096; // 4 KiB

pub struct FrameAllocator {
    free_lists: &'static mut [IntrusiveList<Frame>],
    free_list_bitmap: u64, // TODO: provide bitmap <> free_lits hard sync
    orders: usize,
    memory_layout: &'static mut Layout,
}

impl FrameAllocator {
    pub fn init(layout: &'static mut Layout) -> Self {
        // create frame metadata slice in the frame pool region
        let frame_slice = unsafe {
            core::slice::from_raw_parts_mut(
                layout.frame_pool.start().as_mut_ptr::<Frame>(),
                layout.num_frames(),
            )
        };

        assert_eq!(
            frame_slice.len(),
            layout.num_frames(),
            "Frame slice length doesn't match number of frames"
        );

        frame_slice.iter_mut().for_each(|frame| {
            *frame = Frame::new();
        });

        let orders = (layout.num_frames().ilog2() + 1) as usize;

        // create free intrusive list for each order in the frame allocator metadata region
        let free_lists = unsafe {
            core::slice::from_raw_parts_mut(
                layout
                    .frame_allocator_metadata
                    .start()
                    .as_mut_ptr::<IntrusiveList<Frame>>(),
                orders,
            )
        };

        assert_eq!(
            free_lists.len(),
            orders,
            "Free list count doesn't match orders"
        );

        free_lists.iter_mut().for_each(|list| {
            *list = IntrusiveList::new();
        });

        let mut current_free_address = layout.free_memory.start();
        let mut frames_left = layout.free_memory.size() / BASE_SIZE;

        let mut free_list_bitmap: u64 = 0;

        // greedy algorithm to distribute free memory blocks into free lists
        // starting from the highest order memory block available
        while frames_left > 0 {
            let largest_block_order = frames_left.ilog2();
            let largest_block_frames = 1 << largest_block_order;
            let largest_block_bytes = largest_block_frames * BASE_SIZE;

            let head_frame_idx = (current_free_address - layout.ram.start()) / BASE_SIZE;
            let head_frame = &mut frame_slice[head_frame_idx];

            head_frame.set_order(largest_block_order as u8);

            // set the frame with correspondng order as a head of the ordered free list
            free_lists[largest_block_order as usize].push_front(NonNull::from(head_frame));

            free_list_bitmap |= 1 << largest_block_order; // set the bit for this order

            frames_left -= largest_block_frames;
            current_free_address += largest_block_bytes;
        }

        assert_eq!(frames_left, 0, "Not all frames were initialized");
        assert_eq!(
            current_free_address,
            layout.free_memory.end(),
            "Uninitialized free memory detected"
        );

        FrameAllocator {
            free_lists,
            free_list_bitmap,
            orders,
            memory_layout: layout,
        }
    }

    pub fn order_from_size(&self, size: usize) -> u8 {
        if size == 0 {
            return 0;
        }
        let frames = size.div_ceil(BASE_SIZE); // round up
        frames.next_power_of_two().ilog2() as u8
    }

    pub fn alloc(&mut self, size: usize) -> NonNull<Frame> {
        let order = self.order_from_size(size);
        // TODO: consider invariants for order

        self.alloc_order(order)
    }

    pub fn alloc_order(&mut self, order: u8) -> NonNull<Frame> {
        match self.prepare_block(order) {
            Some(mut head_frame) => {
                let frame = unsafe { head_frame.as_mut() };
                frame.set_state(State::Allocated);

                head_frame
            }
            None => {
                // TODO: handle oom properly
                panic!(
                    "Out Of Memory: no free blocks available for order {}",
                    order
                );
            }
        }
    }

    fn prepare_block(&mut self, requested_order: u8) -> Option<NonNull<Frame>> {
        // create a mask with orders >= requested_order
        let suitable_orders_mask = !((1 << requested_order) - 1);

        // find available blocks in the free list bitmap
        let available_orders = self.free_list_bitmap & suitable_orders_mask;
        if available_orders == 0 {
            return None; // no block found
        }

        // find the smallest suitable order available
        let found_order = available_orders.trailing_zeros() as u8;

        let list = &mut self.free_lists[found_order as usize];
        let mut block_to_split = list.pop_front().unwrap();
        if list.is_empty() {
            self.free_list_bitmap &= !(1 << found_order); // clear the bit
        }

        // split the block down until it fits the requested order
        for current_order in (requested_order..found_order).rev() {
            let block_addr = self
                .memory_layout
                .frame_ref_to_address(unsafe { block_to_split.as_ref() });

            let buddy_offset = (1 << current_order) * BASE_SIZE;
            let buddy_addr = block_addr + buddy_offset;
            let buddy_head_frame = self.memory_layout.address_to_frame_ref(buddy_addr);

            // downgrade blocks order, i.e `split`
            unsafe { block_to_split.as_mut().set_order(current_order) };
            buddy_head_frame.set_order(current_order);

            self.free_lists[current_order as usize].push_front(NonNull::from(buddy_head_frame));
            self.free_list_bitmap |= 1 << current_order; // set the bit for the downgraded order
        }

        Some(block_to_split)
    }

    // FIXME
    pub fn dealloc(&mut self, head_frame: NonNull<Frame>) {
        todo!("Implement deallocation");
    }
}
