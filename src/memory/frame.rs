use crate::collections::{DoublyLinkable, SinglyLinkable};
use core::ptr::NonNull;

pub const BASE_SIZE: usize = 4096; // 4 KiB
const PER_CPU_CACHE_FRAMES: usize = 16;

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

unsafe impl SinglyLinkable for Frame {
    fn next(&self) -> Option<NonNull<Self>> {
        self.next
    }

    fn set_next(&mut self, next: Option<NonNull<Self>>) {
        self.next = next;
    }
}

unsafe impl DoublyLinkable for Frame {
    fn prev(&self) -> Option<NonNull<Self>> {
        self.prev
    }

    fn set_prev(&mut self, prev: Option<NonNull<Self>>) {
        self.prev = prev;
    }
}
