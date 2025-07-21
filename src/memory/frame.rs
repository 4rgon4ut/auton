use crate::collections::{DoublyLinkable, SinglyLinkable};
use crate::memory::slub::{SizeClassManager, Slot};
use core::alloc::Layout;
use core::mem::ManuallyDrop;
use core::ptr::NonNull;

pub const BASE_SIZE: usize = 4096; // 4 KiB
pub const BASE_SIZE_LAYOUT: Layout =
    unsafe { Layout::from_size_align_unchecked(BASE_SIZE, BASE_SIZE) };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Free,
    Allocated,
    Slab,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SlabInfo {
    pub cache: NonNull<SizeClassManager>,
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
    pub slab: ManuallyDrop<SlabInfo>,
    pub buddy: ManuallyDrop<BuddyInfo>,
}

pub struct Frame {
    data: FrameData,
    state: State,

    order: u8,
}

impl Frame {
    pub const fn new() -> Self {
        Frame {
            data: FrameData {
                buddy: ManuallyDrop::new(BuddyInfo {
                    next: None,
                    prev: None,
                }),
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

    pub fn convert_to_slab(
        &mut self,
        cache_ptr: NonNull<SizeClassManager>,
        slots_head: Option<NonNull<Slot>>,
    ) {
        self.state = State::Slab;
        self.data.slab = ManuallyDrop::new(SlabInfo {
            cache: cache_ptr,
            next_slot: slots_head,
            in_use_count: 0,
        });
    }

    pub fn slab_info(&self) -> &SlabInfo {
        debug_assert!(
            matches!(self.state, State::Slab),
            "Attempted to access slab info on a non-slab frame"
        );
        unsafe { &self.data.slab }
    }

    pub fn slab_info_mut(&mut self) -> &mut SlabInfo {
        debug_assert!(
            matches!(self.state, State::Slab),
            "Attempted to access slab info on a non-slab frame"
        );
        unsafe { &mut self.data.slab }
    }

    pub fn buddy_info(&self) -> &BuddyInfo {
        debug_assert!(
            !matches!(self.state, State::Slab),
            "Attempted to access buddy info on a slab frame"
        );
        unsafe { &self.data.buddy }
    }

    pub fn buddy_info_mut(&mut self) -> &mut BuddyInfo {
        debug_assert!(
            !matches!(self.state, State::Slab),
            "Attempted to access buddy info on a slab frame"
        );
        unsafe { &mut self.data.buddy }
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl SinglyLinkable for Frame {
    fn next(&self) -> Option<NonNull<Self>> {
        self.buddy_info().next
    }

    fn set_next(&mut self, next: Option<NonNull<Self>>) {
        debug_assert!(matches!(self.state, State::Free));
        self.buddy_info_mut().next = next;
    }
}

unsafe impl DoublyLinkable for Frame {
    fn prev(&self) -> Option<NonNull<Self>> {
        self.buddy_info().prev
    }

    fn set_prev(&mut self, prev: Option<NonNull<Self>>) {
        debug_assert!(matches!(self.state, State::Free));
        self.buddy_info_mut().prev = prev;
    }
}
