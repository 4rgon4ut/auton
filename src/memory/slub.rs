use crate::collections::{DoublyLinkedList, SinglyLinkable};
use crate::memory::frame::{BASE_SIZE, Frame};
use crate::memory::hart_cache::FillToTarget;
use crate::memory::hart_cache::{HartCache, MAX_HARTS};
use crate::sync::Spinlock;

use core::ptr::NonNull;

const DEFAULT_CACHE_SIZE: usize = 16;

pub struct Slot {
    next: Option<NonNull<Slot>>,
}

unsafe impl SinglyLinkable for Slot {
    fn next(&self) -> Option<NonNull<Self>> {
        self.next
    }

    fn set_next(&mut self, next: Option<NonNull<Self>>) {
        self.next = next;
    }
}

pub struct SizeClassManager {
    hart_caches: [HartCache<Slot, FillToTarget>; MAX_HARTS],

    partial_slabs: Spinlock<DoublyLinkedList<Frame>>,
    empty_slabs: Spinlock<DoublyLinkedList<Frame>>,

    object_size: usize,
}

const MIN_SLOT_SIZE: usize = 8;
const MAX_SLOT_SIZE: usize = BASE_SIZE / 2;

const MIN_SLOT_ORDER: usize = MIN_SLOT_SIZE.ilog2() as usize;
const MAX_SLOT_ORDER: usize = MAX_SLOT_SIZE.ilog2() as usize;

const NUM_CACHES: usize = MAX_SLOT_ORDER - MIN_SLOT_ORDER + 1;

pub struct SlubAllocator {}

impl SlubAllocator {
    pub fn new() -> Self {
        Self {}
    }
}
