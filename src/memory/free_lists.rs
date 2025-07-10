use crate::collections::IntrusiveList;
use crate::memory::frame::Frame;
use core::ptr::NonNull;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct Bitmap(u64);

impl Bitmap {
    #[inline]
    pub const fn new() -> Self {
        Self(0)
    }

    /// sets the bit corresponding to the given order
    #[inline]
    pub fn set(&mut self, order: u8) {
        self.0 |= 1 << order;
    }

    /// clears the bit corresponding to the given order
    #[inline]
    pub fn clear(&mut self, order: u8) {
        self.0 &= !(1 << order);
    }

    /// finds the first available order great than or equal to `requested_order`
    #[inline]
    pub fn find_first_set_from(&self, requested_order: u8) -> Option<u8> {
        // create a mask to ignore orders smaller than requested
        let suitable_mask = !((1 << requested_order) - 1);

        // find >= orders
        let suitable_blocks = self.0 & suitable_mask;

        if suitable_blocks == 0 {
            None
        } else {
            // return the smallest suitable
            Some(suitable_blocks.trailing_zeros() as u8)
        }
    }
}

pub struct FreeLists {
    lists: &'static mut [IntrusiveList<Frame>],
    bitmap: Bitmap,
}

impl FreeLists {
    #[inline]
    pub fn new(lists: &'static mut [IntrusiveList<Frame>]) -> Self {
        Self {
            lists,
            bitmap: Bitmap::new(),
        }
    }

    /// pushes a frame onto the front of the correct free list
    #[inline]
    pub fn push_frame(&mut self, frame: NonNull<Frame>) {
        let order = unsafe { frame.as_ref().order() };
        self.lists[order as usize].push_front(frame);
        self.bitmap.set(order);
    }

    /// pops a frame from the front of the list for a given order
    #[inline]
    pub fn pop_frame(&mut self, order: u8) -> Option<NonNull<Frame>> {
        let frame = self.lists[order as usize].pop_front()?;
        if self.lists[order as usize].is_empty() {
            self.bitmap.clear(order);
        }
        Some(frame)
    }

    #[inline]
    pub fn remove_frame(&mut self, frame: NonNull<Frame>) {
        let order = unsafe { frame.as_ref().order() };
        self.lists[order as usize].remove(frame);
        if self.lists[order as usize].is_empty() {
            self.bitmap.clear(order);
        }
    }

    /// finds the first available order that is greater than or equal to `requested_order`
    #[inline]
    pub fn find_first_free_from(&self, from_order: u8) -> Option<u8> {
        self.bitmap.find_first_set_from(from_order)
    }
}
