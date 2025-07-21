use crate::collections::{SinglyLinkable, SinglyLinkedList};
use core::ptr::NonNull;

pub const MAX_HARTS: usize = 12; // TODO: make dynamic

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
pub struct HartCache<T: SinglyLinkable, S: CacheStrategy> {
    items: SinglyLinkedList<T>,
    strategy: S,
    target_size: usize,
}

impl<T: SinglyLinkable, S: CacheStrategy> HartCache<T, S> {
    pub fn new(target_size: usize, strategy: S) -> Self {
        Self {
            items: SinglyLinkedList::new(),
            strategy,
            target_size,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.len() >= self.target_size()
    }

    #[inline]
    pub fn target_size(&self) -> usize {
        self.target_size
    }

    #[inline]
    pub fn push(&mut self, item: NonNull<T>) {
        self.items.push_front(item);
    }

    #[inline]
    pub fn pop(&mut self) -> Option<NonNull<T>> {
        self.items.pop_front()
    }

    pub fn drain(&mut self) -> impl Iterator<Item = NonNull<T>> {
        self.items.drain(self.drain_amount())
    }

    #[inline]
    pub fn refill_amount(&self) -> usize {
        self.strategy.refill_amount(self.target_size(), self.len())
    }

    #[inline]
    pub fn drain_amount(&self) -> usize {
        self.strategy.drain_amount(self.target_size(), self.len())
    }

    #[inline]
    pub fn grow(&mut self) {
        self.target_size = self.strategy.increase_target(self.target_size)
    }

    #[inline]
    pub fn shrink(&mut self) {
        self.target_size = self.strategy.decrease_target(self.target_size)
    }
}

pub trait CacheStrategy {
    fn refill_amount(&self, target_size: usize, current_len: usize) -> usize;

    fn drain_amount(&self, target_size: usize, current_len: usize) -> usize;

    fn decrease_target(&self, target_size: usize) -> usize;

    fn increase_target(&self, target_size: usize) -> usize;
}

pub struct Quartering;

const QUARTERING_DENOMINATOR: usize = 4;

impl CacheStrategy for Quartering {
    #[inline]
    fn refill_amount(&self, target_size: usize, _current_len: usize) -> usize {
        (target_size / QUARTERING_DENOMINATOR).max(1)
    }

    #[inline]
    fn drain_amount(&self, target_size: usize, current_len: usize) -> usize {
        (target_size / QUARTERING_DENOMINATOR).min(current_len)
    }

    #[inline]
    fn decrease_target(&self, target_size: usize) -> usize {
        target_size / QUARTERING_DENOMINATOR
    }

    #[inline]
    fn increase_target(&self, target_size: usize) -> usize {
        target_size * QUARTERING_DENOMINATOR
    }
}

pub struct Greedy;

impl CacheStrategy for Greedy {
    #[inline]
    fn refill_amount(&self, target_size: usize, current_len: usize) -> usize {
        target_size.saturating_sub(current_len)
    }

    #[inline]
    fn drain_amount(&self, target_size: usize, current_len: usize) -> usize {
        if current_len > 2 * target_size {
            current_len / 2
        } else {
            0
        }
    }

    #[inline]
    fn decrease_target(&self, target_size: usize) -> usize {
        target_size
    }

    #[inline]
    fn increase_target(&self, target_size: usize) -> usize {
        target_size
    }
}
