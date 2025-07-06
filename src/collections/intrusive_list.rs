use core::marker::PhantomData;
use core::ptr::NonNull;

/// A trait for objects that can be part of an `IntrusiveList`.
///
/// # Safety
///
/// The implementor of this trait must guarantee that the `next`, `prev`,
/// `set_next`, and `set_prev` methods exclusively access and modify the
/// internal pointers for the intrusive list and do not perform any other
/// logic. The integrity of the list relies on these methods being implemented
/// correctly.
pub unsafe trait Linkable {
    /// Returns a raw pointer to the next element in the list.
    fn next(&self) -> Option<NonNull<Self>>;

    /// Returns a raw pointer to the previous element in the list.
    fn prev(&self) -> Option<NonNull<Self>>;

    /// Sets the raw pointer to the next element in the list.
    fn set_next(&mut self, next: Option<NonNull<Self>>);

    /// Sets the raw pointer to the previous element in the list.
    fn set_prev(&mut self, prev: Option<NonNull<Self>>);
}

/// A doubly-linked list that is "intrusive."
///
/// This means that the nodes of the list are stored directly within the
/// elements they contain, rather than being allocated separately.
///
/// The user is responsible for managing the memory of the nodes.
pub struct IntrusiveList<T: Linkable> {
    head: Option<NonNull<T>>,
    tail: Option<NonNull<T>>,
    len: usize,
    phantom: PhantomData<*const T>,
}

impl<T: Linkable> IntrusiveList<T> {
    /// Creates a new, empty `IntrusiveList`.
    ///
    /// # Examples
    ///
    /// ```
    /// use intrusive_list::IntrusiveList;
    /// let list: IntrusiveList<MyNode> = IntrusiveList::new();
    /// ```
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
            phantom: PhantomData,
        }
    }

    /// Returns the number of elements in the list.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the list contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a reference to the first element of the list, or `None` if it is empty.
    pub fn front(&self) -> Option<&T> {
        // SAFETY: If `self.head` is `Some`, it is a valid pointer to a `T`.
        self.head.map(|node| unsafe { node.as_ref() })
    }

    /// Returns a mutable reference to the first element of the list, or `None` if it is empty.
    pub fn front_mut(&mut self) -> Option<&mut T> {
        // SAFETY: If `self.head` is `Some`, it is a valid pointer to a `T`.
        // The mutable borrow of `self` ensures exclusive access.
        self.head.map(|mut node| unsafe { node.as_mut() })
    }

    /// Returns a reference to the last element of the list, or `None` if it is empty.
    pub fn back(&self) -> Option<&T> {
        // SAFETY: If `self.tail` is `Some`, it is a valid pointer to a `T`.
        self.tail.map(|node| unsafe { node.as_ref() })
    }

    /// Returns a mutable reference to the last element of the list, or `None` if it is empty.
    pub fn back_mut(&mut self) -> Option<&mut T> {
        // SAFETY: If `self.tail` is `Some`, it is a valid pointer to a `T`.
        // The mutable borrow of `self` ensures exclusive access.
        self.tail.map(|mut node| unsafe { node.as_mut() })
    }

    /// Adds an element to the front of the list.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if the node is already part of a list.
    pub fn push_front(&mut self, mut node: NonNull<T>) {
        assert_detached(node);

        // SAFETY: The node pointer is valid and we have exclusive access.
        let node_ref = unsafe { node.as_mut() };

        match self.head {
            Some(mut old_head) => {
                node_ref.set_next(Some(old_head));
                // SAFETY: `old_head` is a valid pointer as it comes from `self.head`.
                // Exclusive access is guaranteed by `&mut self`.
                unsafe {
                    old_head.as_mut().set_prev(Some(node));
                }
            }
            None => {
                self.tail = Some(node);
            }
        }
        self.head = Some(node);
        self.len += 1;
    }

    /// Removes the first element from the list and returns it.
    ///
    /// Returns `None` if the list is empty.
    pub fn pop_front(&mut self) -> Option<NonNull<T>> {
        self.head.map(|mut old_head| {
            // SAFETY: `old_head` is guaranteed to be a valid pointer by the `map`.
            // We have exclusive access via `&mut self`.
            let old_head_ref = unsafe { old_head.as_mut() };

            self.head = old_head_ref.next();

            match self.head {
                Some(mut new_head) => {
                    // SAFETY: `new_head` is the new head of the list, so it's a valid pointer.
                    // We have exclusive access.
                    unsafe {
                        new_head.as_mut().set_prev(None);
                    }
                }
                None => {
                    self.tail = None;
                }
            }
            self.len -= 1;

            // Detach the node from the list completely.
            old_head_ref.set_next(None);
            old_head_ref.set_prev(None);

            old_head
        })
    }

    /// Adds an element to the back of the list.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if the node is already part of a list.
    pub fn push_back(&mut self, mut node: NonNull<T>) {
        assert_detached(node);

        // SAFETY: The node pointer is valid and we have exclusive access.
        let node_ref = unsafe { node.as_mut() };

        match self.tail {
            Some(mut old_tail) => {
                node_ref.set_prev(Some(old_tail));
                // SAFETY: `old_tail` is a valid pointer as it comes from `self.tail`.
                // Exclusive access is guaranteed by `&mut self`.
                unsafe {
                    old_tail.as_mut().set_next(Some(node));
                }
            }
            None => {
                self.head = Some(node);
            }
        }
        self.tail = Some(node);
        self.len += 1;
    }

    /// Removes the last element from the list and returns it.
    ///
    /// Returns `None` if the list is empty.
    pub fn pop_back(&mut self) -> Option<NonNull<T>> {
        self.tail.map(|mut old_tail| {
            // SAFETY: `old_tail` is guaranteed to be a valid pointer by the `map`.
            // We have exclusive access via `&mut self`.
            let old_tail_ref = unsafe { old_tail.as_mut() };

            self.tail = old_tail_ref.prev();

            match self.tail {
                Some(mut new_tail) => {
                    // SAFETY: `new_tail` is valid as it's the new tail of the list.
                    // We have exclusive access.
                    unsafe {
                        new_tail.as_mut().set_next(None);
                    }
                }
                None => {
                    self.head = None;
                }
            }
            self.len -= 1;

            // Detach the node from the list completely.
            old_tail_ref.set_next(None);
            old_tail_ref.set_prev(None);

            old_tail
        })
    }

    /// Returns a `CursorMut` that points to the first element of the list.
    pub fn cursor_mut<'a>(&'a mut self) -> CursorMut<'a, T> {
        CursorMut {
            list: NonNull::from(&mut *self),
            current: self.head,
            phantom: PhantomData,
        }
    }
}

impl<T: Linkable> Default for IntrusiveList<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// A cursor with mutable access to an `IntrusiveList`.
///
/// A `CursorMut` allows for navigation and manipulation of the list.
pub struct CursorMut<'a, T: Linkable> {
    list: NonNull<IntrusiveList<T>>,
    current: Option<NonNull<T>>,
    phantom: PhantomData<&'a mut T>,
}

impl<'a, T: Linkable> CursorMut<'a, T> {
    /// Returns a reference to the element currently pointed to by the cursor.
    pub fn current(&self) -> Option<&T> {
        // SAFETY: If `self.current` is `Some`, it points to a valid node
        // within the list, and the lifetime `'a` ensures it doesn't outlive the list.
        self.current.map(|node_ptr| unsafe { node_ptr.as_ref() })
    }

    /// Returns a mutable reference to the element currently pointed to by the cursor.
    pub fn current_mut(&mut self) -> Option<&mut T> {
        // SAFETY: If `self.current` is `Some`, it points to a valid node.
        // The `&mut self` borrow ensures exclusive access.
        self.current
            .map(|mut node_ptr| unsafe { node_ptr.as_mut() })
    }

    /// Returns `true` if the cursor is "dangling" (pointing to no element).
    pub fn is_dangling(&self) -> bool {
        self.current.is_none()
    }

    /// Moves the cursor to the next element and returns a mutable reference to it.
    pub fn move_next(&mut self) -> Option<&mut T> {
        let next = self.current().and_then(|node| node.next());

        self.current = next;

        self.current_mut()
    }

    /// Moves the cursor to the previous element and returns a mutable reference to it.
    pub fn move_prev(&mut self) -> Option<&mut T> {
        let prev = self.current().and_then(|node| node.prev());

        self.current = prev;

        self.current_mut()
    }

    /// Removes the current element from the list and returns it.
    ///
    /// The cursor is moved to the next element. If the removed element was the
    /// last one, the cursor becomes dangling.
    pub fn remove_current(&mut self) -> Option<NonNull<T>> {
        let mut current_ptr = self.current.take()?;

        // SAFETY: `self.list` is a valid pointer to the list, and the lifetime `'a`
        // guarantees it's still alive. `&mut self` ensures exclusive access.
        let list = unsafe { self.list.as_mut() };
        // SAFETY: `current_ptr` was just taken from `self.current`, so it's a valid pointer.
        let current_node = unsafe { current_ptr.as_mut() };

        let prev = current_node.prev();
        let next = current_node.next();

        match prev {
            // SAFETY: `prev_node` is a valid pointer from `current_node`.
            Some(mut prev_node) => unsafe { prev_node.as_mut().set_next(next) },
            None => list.head = next,
        }

        match next {
            // SAFETY: `next_node` is a valid pointer from `current_node`.
            Some(mut next_node) => unsafe { next_node.as_mut().set_prev(prev) },
            None => list.tail = prev,
        }

        list.len -= 1;

        // Detach the node.
        current_node.set_next(None);
        current_node.set_prev(None);

        self.current = next;

        Some(current_ptr)
    }

    /// Inserts a new node before the current element.
    ///
    /// If the cursor is dangling, the node is inserted at the back of the list.
    /// The cursor is moved to point to the newly inserted node.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if the new node is already part of a list.
    pub fn insert_before(&mut self, mut new_node: NonNull<T>) {
        assert_detached(new_node);

        // SAFETY: `self.list` is a valid pointer.
        let list = unsafe { self.list.as_mut() };

        match self.current {
            Some(mut current_node) => {
                let prev_node = unsafe { current_node.as_mut().prev() };

                // SAFETY: `new_node` and `current_node` are valid pointers.
                // Links are being updated to insert the new node.
                unsafe {
                    new_node.as_mut().set_next(Some(current_node));
                    new_node.as_mut().set_prev(prev_node);
                    current_node.as_mut().set_prev(Some(new_node));
                }

                match prev_node {
                    // SAFETY: `p` is a valid pointer.
                    Some(mut p) => unsafe { p.as_mut().set_next(Some(new_node)) },
                    None => list.head = Some(new_node),
                }
                list.len += 1;
            }
            None => {
                // If cursor is dangling, push to the back.
                list.push_back(new_node);
            }
        }
        self.current = Some(new_node);
    }

    /// Inserts a new node after the current element.
    ///
    /// If the cursor is dangling, the node is inserted at the front of the list.
    /// The cursor is moved to point to the newly inserted node.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if the new node is already part of a list.
    pub fn insert_after(&mut self, mut new_node: NonNull<T>) {
        assert_detached(new_node);

        // SAFETY: `self.list` is a valid pointer.
        let list = unsafe { self.list.as_mut() };

        match self.current {
            Some(mut current_node) => {
                let next_node = unsafe { current_node.as_mut().next() };

                // SAFETY: `new_node` and `current_node` are valid pointers.
                // Links are updated to insert the new node.
                unsafe {
                    new_node.as_mut().set_next(next_node);
                    new_node.as_mut().set_prev(Some(current_node));
                    current_node.as_mut().set_next(Some(new_node));
                }

                match next_node {
                    // SAFETY: `n` is a valid pointer.
                    Some(mut n) => unsafe { n.as_mut().set_prev(Some(new_node)) },
                    None => list.tail = Some(new_node),
                }
                list.len += 1;
            }
            None => {
                // If cursor is dangling, push to the front.
                list.push_front(new_node);
            }
        }
        self.current = Some(new_node);
    }

    /// Splits the list into two after the current element.
    ///
    /// Returns a new `IntrusiveList` containing all elements after the current one.
    /// The current element becomes the new tail of the original list.
    /// If the cursor is at the tail, an empty list is returned.
    pub fn split_after(&mut self) -> IntrusiveList<T> {
        let Some(mut current_ptr) = self.current else {
            return IntrusiveList::new();
        };

        // SAFETY: `current_ptr` is valid.
        let Some(mut new_head_ptr) = (unsafe { current_ptr.as_ref().next() }) else {
            return IntrusiveList::new();
        };

        // SAFETY: `self.list` is a valid pointer.
        let list = unsafe { self.list.as_mut() };
        let old_tail = list.tail;

        // SAFETY: Pointers are valid. We are severing the list.
        unsafe {
            new_head_ptr.as_mut().set_prev(None);
            current_ptr.as_mut().set_next(None);
            list.tail = Some(current_ptr);
        }

        // Count moved nodes to update lengths correctly.
        let mut moved_nodes_count = 0;
        let mut temp_node = Some(new_head_ptr);
        while let Some(node) = temp_node {
            moved_nodes_count += 1;
            // SAFETY: `node` is valid within this loop.
            temp_node = unsafe { node.as_ref().next() };
        }

        list.len -= moved_nodes_count;

        IntrusiveList {
            head: Some(new_head_ptr),
            tail: old_tail,
            len: moved_nodes_count,
            phantom: PhantomData,
        }
    }

    /// Moves all elements from another list and inserts them after the current element.
    ///
    /// If the cursor is dangling, the elements are inserted at the end of the list.
    /// The `other` list will be empty after this operation.
    pub fn splice_after(&mut self, other: &mut IntrusiveList<T>) {
        if other.is_empty() {
            return;
        }

        let mut other_head = other.head.take().unwrap();
        let mut other_tail = other.tail.take().unwrap();

        let other_len = other.len;
        other.len = 0;

        // SAFETY: `self.list` is a valid pointer.
        let list = unsafe { self.list.as_mut() };

        match self.current {
            Some(mut current_ptr) => {
                // SAFETY: `current_ptr` is valid.
                let original_next = unsafe { current_ptr.as_ref().next() };

                // SAFETY: Pointers are valid. Splicing the lists together.
                unsafe {
                    current_ptr.as_mut().set_next(Some(other_head));
                    other_head.as_mut().set_prev(Some(current_ptr));
                }

                match original_next {
                    Some(mut next_ptr) => {
                        // SAFETY: Pointers are valid.
                        unsafe {
                            other_tail.as_mut().set_next(Some(next_ptr));
                            next_ptr.as_mut().set_prev(Some(other_tail));
                        }
                    }
                    None => {
                        list.tail = Some(other_tail);
                    }
                }
            }
            None => match list.tail {
                Some(mut old_tail) => {
                    // SAFETY: Pointers are valid.
                    unsafe {
                        old_tail.as_mut().set_next(Some(other_head));
                        other_head.as_mut().set_prev(Some(old_tail));
                    }
                    list.tail = Some(other_tail);
                }
                None => {
                    // The list was empty, so `other` becomes the new list.
                    list.head = Some(other_head);
                    list.tail = Some(other_tail);
                }
            },
        }
        list.len += other_len;
    }
}

/// Asserts that a node's pointers are `None`.
///
/// This is a sanity check to ensure a node isn't already in a list
/// before an operation that would insert it.
#[inline]
fn assert_detached<T: Linkable>(node: NonNull<T>) {
    // SAFETY: The caller must ensure `node` is a valid pointer.
    // This function is only used in debug builds for internal consistency checks.
    assert!(
        unsafe { node.as_ref().next().is_none() && node.as_ref().prev().is_none() },
        "Node is already in a list"
    );
}
