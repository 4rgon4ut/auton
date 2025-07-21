use core::marker::PhantomData;
use core::ptr::NonNull;

/// A trait for objects that can be part of a `SinglyLinkedList`.
///
/// # Safety
/// The implementor must guarantee these methods only access the internal
/// pointer for the list and do not have side effects.
pub unsafe trait SinglyLinkable {
    /// Returns a raw pointer to the next element in the list.
    fn next(&self) -> Option<NonNull<Self>>;

    /// Sets the raw pointer to the next element in the list.
    fn set_next(&mut self, next: Option<NonNull<Self>>);
}

/// An intrusive, singly-linked list.
///
/// This list is optimized for stack-like (LIFO) operations, providing
/// O(1) `push_front` and `pop_front`. It does not keep a tail pointer
/// and is therefore highly memory-efficient.
pub struct SinglyLinkedList<T: SinglyLinkable> {
    head: Option<NonNull<T>>,
    len: usize,
    phantom: PhantomData<*const T>,
}

impl<T: SinglyLinkable> SinglyLinkedList<T> {
    pub const fn new() -> Self {
        Self {
            head: None,
            len: 0,
            phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn front(&self) -> Option<&T> {
        self.head.map(|node| unsafe { node.as_ref() })
    }

    pub fn push_front(&mut self, mut node: NonNull<T>) {
        debug_assert!(
            unsafe { node.as_ref().next().is_none() },
            "Node is already linked"
        );

        let node_ref = unsafe { node.as_mut() };
        node_ref.set_next(self.head);
        self.head = Some(node);
        self.len += 1;
    }

    pub fn pop_front(&mut self) -> Option<NonNull<T>> {
        self.head.map(|mut old_head| {
            let old_head_ref = unsafe { old_head.as_mut() };
            self.head = old_head_ref.next();
            self.len -= 1;

            old_head_ref.set_next(None);
            old_head
        })
    }

    pub fn drain(&mut self, amount: usize) -> impl Iterator<Item = NonNull<T>> {
        let drained_list = if amount == 0 {
            SinglyLinkedList::new()
        } else if amount >= self.len {
            core::mem::take(self)
        } else {
            let mut tail_of_drained = self.head.unwrap();
            for _ in 0..(amount - 1) {
                tail_of_drained = unsafe { tail_of_drained.as_ref().next().unwrap() };
            }

            let new_head_for_self = unsafe { tail_of_drained.as_ref().next() };
            let old_head = self.head;

            self.head = new_head_for_self;
            self.len -= amount;

            unsafe {
                tail_of_drained.as_mut().set_next(None);
            }

            SinglyLinkedList {
                head: old_head,
                len: amount,
                phantom: PhantomData,
            }
        };

        drained_list.into_iter()
    }

    pub fn clear(&mut self) {
        core::mem::take(self);
    }
}

impl<T: SinglyLinkable> Default for SinglyLinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct IntoIter<T: SinglyLinkable> {
    list: SinglyLinkedList<T>,
}

impl<T: SinglyLinkable> IntoIterator for SinglyLinkedList<T> {
    type Item = NonNull<T>;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { list: self }
    }
}

impl<T: SinglyLinkable> Iterator for IntoIter<T> {
    type Item = NonNull<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_front()
    }
}
