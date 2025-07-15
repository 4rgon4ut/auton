use crate::collections::{SinglyLinkable, SinglyLinkedList};
use core::ptr::NonNull;

/// A per-hart (per-CPU) cache of free memory frames.
///
/// # Cache Line Alignment
///
/// This struct is aligned to a 64-byte boundary using `#[repr(align(64))]`.
/// In a multi-core system, an array of `HartCache` structs is created, one for each core.
/// Without this alignment, multiple caches could reside on the same 64-byte CPU cache line.
///
/// This would lead to **false sharing**: when one core modifies its cache, the cache
/// coherency protocol would invalidate the line for all other cores, even though
/// they were accessing different data. This constant invalidation causes severe
// performance degradation.
///
/// Aligning the struct ensures that each `HartCache` occupies its own cache line,
/// allowing each core to access its local cache without interfering with others.
#[repr(align(64))]
#[derive(Default)]
pub struct HartCache<T: SinglyLinkable> {
    items: SinglyLinkedList<T>,
    target_size: usize,
}

impl<T: SinglyLinkable> HartCache<T> {
    pub fn new(target_size: usize) -> Self {
        Self {
            items: SinglyLinkedList::new(),
            target_size,
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.len() >= self.target_size()
    }

    pub fn target_size(&self) -> usize {
        self.target_size
    }

    pub fn set_target_size(&mut self, target_size: usize) {
        self.target_size = target_size;
    }

    pub fn push(&mut self, item: NonNull<T>) {
        self.items.push_front(item);
    }

    pub fn pop(&mut self) -> Option<NonNull<T>> {
        self.items.pop_front()
    }
}
