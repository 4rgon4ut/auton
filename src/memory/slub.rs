use crate::cpu::current_hart_id;
use crate::memory::frame::{BASE_SIZE, BASE_SIZE_LAYOUT, Frame};
use crate::memory::hart_cache::{Greedy, HartCache, MAX_HARTS};
use crate::memory::{FrameAllocator, frame_allocator, pmem_map};
use crate::sync::{OnceLock, Spinlock};
use crate::{
    collections::{DoublyLinkedList, SinglyLinkable},
    memory::PhysicalAddress,
};

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr;
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

const MIN_HART_CACHE_TARGET: usize = 8;
const MAX_HART_CACHE_TARGET: usize = 128;
const EMPTY_SLABS_CAP: usize = 4; // TODO: Make dynamic based on memory pressure

pub struct SizeClassManager {
    hart_caches: [UnsafeCell<HartCache<Slot, Greedy>>; MAX_HARTS], // TODO: make dynamic based on number of harts

    partial_slabs: Spinlock<DoublyLinkedList<Frame>>,
    empty_slabs: Spinlock<DoublyLinkedList<Frame>>,

    object_size: usize,
    slots_per_slab: usize,
}

impl SizeClassManager {
    pub fn new(num_harts: usize, object_size: usize) -> Self {
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
            } else if let Some(slab) = self.empty_slabs.lock().pop_front() {
                slab
            } else {
                self.create_new_slab()?
            };

            let slab_ref = unsafe { &mut slab_to_process.as_mut() };
            let mut slab_info = slab_ref.lock_slab_info();

            while amount_to_refill > 0 {
                match slab_info.next_slot {
                    Some(slot_ptr) => {
                        let slot = unsafe { slot_ptr.as_ref() };
                        slab_info.next_slot = slot.next;

                        cache.push(slot_ptr);
                        slab_info.in_use_count += 1;

                        amount_to_refill -= 1;
                    }
                    None => break,
                }
            }

            if slab_info.next_slot.is_some() {
                self.partial_slabs.lock().push_front(slab_to_process);
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

        let pm_map = pmem_map();

        cache.drain().for_each(|mut slot_ptr| {
            let mut frame_ptr =
                pm_map.address_to_frame_ptr(PhysicalAddress::from(slot_ptr.as_ptr() as usize));

            let frame = unsafe { frame_ptr.as_mut() };
            let mut slab_info = frame.lock_slab_info();
            let slot = unsafe { slot_ptr.as_mut() };

            let was_full = slab_info.in_use_count == self.slots_per_slab;

            slot.next = slab_info.next_slot;
            slab_info.next_slot = Some(slot_ptr);
            slab_info.in_use_count -= 1;

            if was_full {
                // now partial
                self.partial_slabs.lock().push_front(frame_ptr);
            } else if slab_info.in_use_count == 0 {
                // now empty
                self.partial_slabs.lock().remove(frame_ptr);

                let mut empty_slabs = self.empty_slabs.lock();
                empty_slabs.push_front(frame_ptr);

                if empty_slabs.len() >= EMPTY_SLABS_CAP
                    && let Some(oldest_slab) = empty_slabs.pop_back()
                {
                    drop(empty_slabs);
                    frame_allocator().dealloc(oldest_slab.cast(), BASE_SIZE_LAYOUT);
                }
            }
        });
    }
}

const SIZE_CLASSES: [usize; 9] = [8, 16, 32, 64, 128, 256, 512, 1024, 2048];
const NUM_CACHES: usize = SIZE_CLASSES.len();

// TODO: consider Poisoning/Red-zoning
pub struct SlubAllocator {
    size_classes: [SizeClassManager; NUM_CACHES],
}

impl SlubAllocator {
    pub fn new(num_harts: usize) -> Self {
        Self {
            size_classes: core::array::from_fn(|i| {
                SizeClassManager::new(num_harts, SIZE_CLASSES[i])
            }),
        }
    }

    fn find_size_class(&self, layout: Layout) -> Option<&SizeClassManager> {
        self.size_classes
            .iter()
            .find(|class| class.object_size >= layout.size())
    }
}

pub struct KernelAllocator(OnceLock<SlubAllocator>);

#[allow(clippy::new_without_default)]
impl KernelAllocator {
    pub const fn new() -> Self {
        Self(OnceLock::new())
    }
}

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Some(slub_allocator) = self.0.get() {
            slub_allocator
                .find_size_class(layout)
                .and_then(|class_manager| class_manager.alloc())
                .map(|non_null_ptr| non_null_ptr.as_ptr())
                .unwrap_or(ptr::null_mut())
        } else {
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if ptr.is_null() {
            return;
        }

        let slub_allocator = self.0.get().expect("SlubAllocator not initialized");

        if let Some(class_manager) = slub_allocator.find_size_class(layout) {
            // checked for null above
            let non_null_ptr = unsafe { NonNull::new_unchecked(ptr) };
            class_manager.dealloc(non_null_ptr);
        } else {
            // critical error
            panic!(
                "dealloc called with unsupported layout: size={}, align={}",
                layout.size(),
                layout.align()
            );
        }
    }
}

// TODO: double check
unsafe impl Send for SlubAllocator {}
unsafe impl Sync for SlubAllocator {}
