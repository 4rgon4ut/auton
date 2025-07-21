use crate::collections::{DoublyLinkedList, SinglyLinkable};
use crate::cpu::current_hart_id;
use crate::memory::frame::{BASE_SIZE, Frame};
use crate::memory::hart_cache::{Greedy, HartCache, MAX_HARTS};
use crate::memory::{FrameAllocator, frame_allocator, pmem_map};
use crate::sync::Spinlock;

use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::ptr::NonNull;

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
    hart_caches: [UnsafeCell<HartCache<Slot, Greedy>>; MAX_HARTS],

    partial_slabs: Spinlock<DoublyLinkedList<Frame>>,
    empty_slabs: Spinlock<DoublyLinkedList<Frame>>,

    object_size: usize,
    slots_per_slab: usize,
}

const MIN_HART_CACHE_TARGET: usize = 8;
const MAX_HART_CACHE_TARGET: usize = 128;

impl SizeClassManager {
    pub fn new(object_size: usize) -> Self {
        let slots_per_slab = BASE_SIZE / object_size;

        let hart_cache_target = slots_per_slab.clamp(MIN_HART_CACHE_TARGET, MAX_HART_CACHE_TARGET);

        let hart_caches =
            core::array::from_fn(|_| UnsafeCell::new(HartCache::new(hart_cache_target, Greedy)));

        Self {
            hart_caches,
            partial_slabs: Spinlock::new(DoublyLinkedList::new()),
            empty_slabs: Spinlock::new(DoublyLinkedList::new()),
            object_size,
            slots_per_slab,
        }
    }

    #[inline]
    #[allow(clippy::mut_from_ref)]
    fn hart_cache(&self, hart_id: usize) -> &mut HartCache<Slot, Greedy> {
        unsafe { &mut *self.hart_caches[hart_id].get() }
    }

    pub fn alloc(&self) -> Option<NonNull<u8>> {
        let hart_id = current_hart_id();
        let cache = self.hart_cache(hart_id);

        if let Some(slot) = cache.pop() {
            return Some(slot.cast());
        }

        self.refill_hart_cache(hart_id).ok()?;

        cache.pop().map(|slot| slot.cast())
    }

    fn create_new_slab(&self) -> Result<NonNull<Frame>, ()> {
        let mut frame = frame_allocator().alloc_slab().ok_or(())?;
        let frame_ref = unsafe { frame.as_mut() };

        let frame_addr = pmem_map().frame_ref_to_address(frame_ref);

        let start_ptr = frame_addr.as_mut_ptr::<u8>();

        for i in 0..(self.slots_per_slab - 1) {
            unsafe {
                let current_slot_ptr = start_ptr.add(i * self.object_size).cast::<Slot>();
                let next_slot_ptr = start_ptr.add((i + 1) * self.object_size).cast::<Slot>();

                (*current_slot_ptr).next = Some(NonNull::new_unchecked(next_slot_ptr));
            }
        }

        // explicitly set last slot `next` to None in case of stale garbage in provided frame
        unsafe {
            let last_slot_ptr = start_ptr
                .add((self.slots_per_slab - 1) * self.object_size)
                .cast::<Slot>();
            (*last_slot_ptr).next = None;
        }

        let head = NonNull::new(start_ptr.cast::<Slot>());

        frame_ref.convert_to_slab(NonNull::from(self), head);

        Ok(frame)
    }

    fn refill_hart_cache(&self, hart_id: usize) -> Result<(), ()> {
        let cache = self.hart_cache(hart_id);

        let mut amount_to_refill = cache.refill_amount();

        while amount_to_refill > 0 {
            let mut slab_to_process = if let Some(slab) = self.partial_slabs.lock().pop_front() {
                slab
            } else {
                self.create_new_slab()?
            };

            let slab_ref = unsafe { &mut slab_to_process.as_mut() };
            let slab_data = slab_ref.slab_info_mut();

            while amount_to_refill > 0 {
                match slab_data.next_slot {
                    Some(slot_ptr) => {
                        let slot = unsafe { slot_ptr.as_ref() };
                        slab_data.next_slot = slot.next;

                        cache.push(slot_ptr);

                        amount_to_refill -= 1;
                    }
                    None => break,
                }
            }

            if slab_data.next_slot.is_some() {
                self.partial_slabs.lock().push_front(slab_to_process);
            } else {
                self.empty_slabs.lock().push_front(slab_to_process);
            }
        }

        Ok(())
    }

    pub fn dealloc(&self, ptr: NonNull<u8>) {
        let hart_id = current_hart_id();
        let cache = self.hart_cache(hart_id);

        let slot = ptr.cast::<Slot>();

        if !cache.is_full() {
            return cache.push(slot);
        }

        // TODO:
        cache.drain().for_each(|slot| {});
        todo!()
    }
}

const SIZE_CLASSES: [usize; 9] = [8, 16, 32, 64, 128, 256, 512, 1024, 2048];
const NUM_CACHES: usize = SIZE_CLASSES.len();

pub struct SlubAllocator {
    size_classes: [SizeClassManager; NUM_CACHES],
    frame_allocator: &'static FrameAllocator,
}

impl SlubAllocator {
    pub fn new(frame_allocator: &'static FrameAllocator) -> Self {
        Self {
            size_classes: core::array::from_fn(|i| SizeClassManager::new(SIZE_CLASSES[i])),
            frame_allocator,
        }
    }

    fn find_size_class(&self, layout: Layout) -> Option<&SizeClassManager> {
        self.size_classes
            .iter()
            .find(|class| class.object_size >= layout.size())
    }
}
