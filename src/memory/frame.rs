use crate::collections::{DoublyLinkable, SinglyLinkable};
use crate::memory::slub::{SizeClassManager, Slot};
use core::ptr::NonNull;

pub const BASE_SIZE: usize = 4096; // 4 KiB
const PER_CPU_CACHE_FRAMES: usize = 16;

#[derive(Debug, Clone, Copy)]
pub enum State {
    Free,
    Allocated,
    Slab,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SlabInfo {
    pub cache: *const SizeClassManager,
    pub next_slot: Option<NonNull<Slot>>,
    pub in_use_count: u16,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct BuddyInfo {
    pub next: Option<NonNull<Frame>>,
    pub prev: Option<NonNull<Frame>>,
}

#[repr(C)]
pub union FrameData {
    pub slab: SlabInfo,
    pub buddy: BuddyInfo,
}

pub struct Frame {
    pub data: FrameData,

    order: u8,
    state: State,
}

impl Frame {
    pub const fn new() -> Self {
        Frame {
            data: FrameData {
                buddy: BuddyInfo {
                    next: None,
                    prev: None,
                },
            },
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
        unsafe { self.data.buddy.next }
    }

    fn set_next(&mut self, next: Option<NonNull<Self>>) {
        debug_assert!(matches!(self.state, State::Free));
        self.data.buddy.next = next;
    }
}

unsafe impl DoublyLinkable for Frame {
    fn prev(&self) -> Option<NonNull<Self>> {
        unsafe { self.data.buddy.prev }
    }

    fn set_prev(&mut self, prev: Option<NonNull<Self>>) {
        debug_assert!(matches!(self.state, State::Free));
        self.data.buddy.prev = prev;
    }
}
