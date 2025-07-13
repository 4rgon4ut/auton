use crate::collections::{SinglyLinkable, SinglyLinkedList};
use core::array;
use core::ptr::NonNull;

struct Slot {
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

struct Slab {
    next: Option<NonNull<Self>>,
    prev: Option<NonNull<Self>>,

    free_slots: SinglyLinkedList<Slot>,
}

unsafe impl SinglyLinkable for Slab {
    fn next(&self) -> Option<NonNull<Self>> {
        self.next
    }

    fn set_next(&mut self, next: Option<NonNull<Self>>) {
        self.next = next;
    }
}

impl Slab {
    fn new() -> Self {
        Slab {
            next: None,
            prev: None,
            free_slots: SinglyLinkedList::new(),
        }
    }
}

struct Cache {
    slot_size: usize,

    partial_slabs: SinglyLinkedList<Slab>,
    full_slabs: SinglyLinkedList<Slab>,
    empty_slabs: SinglyLinkedList<Slab>,
}

impl Cache {
    pub fn new(slot_size: usize) -> Self {
        Cache {
            slot_size,
            partial_slabs: SinglyLinkedList::new(),
            full_slabs: SinglyLinkedList::new(),
            empty_slabs: SinglyLinkedList::new(),
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
