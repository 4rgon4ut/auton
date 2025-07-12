use crate::collections::{IntrusiveList, Linkable};
use core::array;
use core::ptr::NonNull;

// TODO: I need single-linked list for this
struct Slot {
    next: Option<NonNull<Slot>>,
}

// TODO: I need single-linked list for this
unsafe impl Linkable for Slot {
    fn next(&self) -> Option<NonNull<Self>> {
        self.next
    }

    fn set_next(&mut self, next: Option<NonNull<Self>>) {
        self.next = next;
    }

    fn prev(&self) -> Option<NonNull<Self>> {
        None
    }

    fn set_prev(&mut self, _prev: Option<NonNull<Self>>) {}
}

struct Slab {
    next: Option<NonNull<Self>>,
    prev: Option<NonNull<Self>>,

    free_slots: IntrusiveList<Slot>,
}

unsafe impl Linkable for Slab {
    fn next(&self) -> Option<NonNull<Self>> {
        self.next
    }

    fn set_next(&mut self, next: Option<NonNull<Self>>) {
        self.next = next;
    }

    fn prev(&self) -> Option<NonNull<Self>> {
        self.prev
    }

    fn set_prev(&mut self, prev: Option<NonNull<Self>>) {
        self.prev = prev;
    }
}

impl Slab {
    fn new() -> Self {
        Slab {
            next: None,
            prev: None,
            free_slots: IntrusiveList::new(),
        }
    }
}

struct Cache {
    slot_size: usize,

    partial_slabs: IntrusiveList<Slab>,
    full_slabs: IntrusiveList<Slab>,
    empty_slabs: IntrusiveList<Slab>,
}

impl Cache {
    pub fn new(slot_size: usize) -> Self {
        Cache {
            slot_size,
            partial_slabs: IntrusiveList::new(),
            full_slabs: IntrusiveList::new(),
            empty_slabs: IntrusiveList::new(),
        }
    }
}

const MIN_SLOT_SIZE: u16 = 8;
const MAX_SLOT_SIZE: u16 = 2048;

const MIN_SLOT_ORDER: u32 = MIN_SLOT_SIZE.ilog2();
const MAX_SLOT_ORDER: u32 = MAX_SLOT_SIZE.ilog2();

const NUM_CACHES: usize = (MAX_SLOT_ORDER - MIN_SLOT_ORDER + 1) as usize;

pub struct SlubAllocator {
    caches: [Cache; NUM_CACHES],
}

impl SlubAllocator {
    pub fn new() -> Self {
        Self {
            caches: array::from_fn(|i| {
                let power = MIN_SLOT_ORDER + i as u32;
                let slot_size = 1 << power;
                Cache::new(slot_size as usize)
            }),
        }
    }
}
