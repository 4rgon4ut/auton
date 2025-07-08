use crate::collections::{IntrusiveList, Linkable};
use crate::memory::Layout;
use core::{panic, ptr::NonNull};

#[derive(Debug)]
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
    orders: usize,
    memory_layout: &'static Layout,
}

impl FrameAllocator {
    pub fn init(layout: &'static Layout) -> Self {
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

        while frames_left > 0 {
            let largest_block_order = frames_left.ilog2();
            let largest_block_frames = 1 << largest_block_order;
            let largest_block_bytes = largest_block_frames * BASE_SIZE;

            let head_frame_idx = (current_free_address - layout.ram.start()) / BASE_SIZE;
            let head_frame = &mut frame_slice[head_frame_idx];

            head_frame.set_order(largest_block_order as u8);

            free_lists[largest_block_order as usize].push_front(NonNull::from(head_frame));

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
            orders,
            memory_layout: layout,
        }
    }

    // FIXME
    pub fn alloc() {
        panic!("Frame allocation not implemented yet");
    }

    // FIXME
    pub fn dealloc() {
        panic!("Frame deallocation not implemented yet");
    }
}
