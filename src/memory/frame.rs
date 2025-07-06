use crate::collections::{IntrusiveList, Linkable};
use core::ptr::NonNull;

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

const BASE_SIZE: usize = 4096; // 4 KiB

pub struct FrameAllocator {
    free_lists: &'static mut [IntrusiveList<Frame>],

    start_address: usize,
    size: usize,
}

impl FrameAllocator {
    // pub const fn new() -> Self {
    //     FrameAllocator {}
    // }
}
