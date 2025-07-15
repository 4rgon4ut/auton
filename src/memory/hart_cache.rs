use crate::collections::{SinglyLinkable, SinglyLinkedList};
use core::ptr::NonNull;

#[repr(align(64))]
#[derive(Default)]
pub struct HartCache<T: SinglyLinkable, const TARGET_CACHE_SIZE: usize> {
    items: SinglyLinkedList<T>,
}

impl<T: SinglyLinkable, const TARGET_CACHE_SIZE: usize> HartCache<T, TARGET_CACHE_SIZE> {
    pub const fn new() -> Self {
        Self {
            items: SinglyLinkedList::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.len() >= TARGET_CACHE_SIZE
    }

    pub fn target_size(&self) -> usize {
        TARGET_CACHE_SIZE
    }

    pub fn push(&mut self, item: NonNull<T>) {
        self.items.push_front(item);
    }

    pub fn pop(&mut self) -> Option<NonNull<T>> {
        self.items.pop_front()
    }
}
