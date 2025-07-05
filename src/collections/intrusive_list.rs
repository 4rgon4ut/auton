use core::marker::PhantomData;
use core::ptr::NonNull;

pub trait Linkable {
    fn next(&self) -> Option<NonNull<Self>>;

    fn prev(&self) -> Option<NonNull<Self>>;

    fn set_next(&mut self, next: Option<NonNull<Self>>);

    fn set_prev(&mut self, prev: Option<NonNull<Self>>);
}

pub struct IntrusiveList<T: Linkable> {
    head: Option<NonNull<T>>,
    tail: Option<NonNull<T>>,
    len: usize,
    phantom: PhantomData<*const T>,
}

impl<T: Linkable> IntrusiveList<T> {
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
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

    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.head.map(|mut node| unsafe { node.as_mut() })
    }

    pub fn back(&self) -> Option<&T> {
        self.tail.map(|node| unsafe { node.as_ref() })
    }

    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.tail.map(|mut node| unsafe { node.as_mut() })
    }

    pub fn push_front(&mut self, mut node: NonNull<T>) {
        assert_detached(node);

        let node_ref = unsafe { node.as_mut() };

        match self.head {
            Some(mut old_head) => {
                node_ref.set_next(Some(old_head));
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

    pub fn pop_front(&mut self) -> Option<NonNull<T>> {
        self.head.map(|mut old_head| {
            let old_head_ref = unsafe { old_head.as_mut() };

            self.head = old_head_ref.next();

            match self.head {
                Some(mut new_head) => unsafe {
                    new_head.as_mut().set_prev(None);
                },
                None => {
                    self.tail = None;
                }
            }
            self.len -= 1;

            old_head_ref.set_next(None);
            old_head_ref.set_prev(None);

            old_head
        })
    }

    pub fn push_back(&mut self, mut node: NonNull<T>) {
        assert_detached(node);

        let node_ref = unsafe { node.as_mut() };

        match self.tail {
            Some(mut old_tail) => {
                node_ref.set_prev(Some(old_tail));
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

    pub fn pop_back(&mut self) -> Option<NonNull<T>> {
        self.tail.map(|mut old_tail| {
            let old_tail_ref = unsafe { old_tail.as_mut() };

            self.tail = old_tail_ref.prev();

            match self.tail {
                Some(mut new_tail) => unsafe {
                    new_tail.as_mut().set_next(None);
                },
                None => {
                    self.head = None;
                }
            }
            self.len -= 1;

            old_tail_ref.set_next(None);
            old_tail_ref.set_prev(None);

            old_tail
        })
    }

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

pub struct CursorMut<'a, T: Linkable> {
    list: NonNull<IntrusiveList<T>>,
    current: Option<NonNull<T>>,
    phantom: PhantomData<&'a mut T>,
}

impl<'a, T: Linkable> CursorMut<'a, T> {
    pub fn current(&self) -> Option<&T> {
        self.current.map(|node_ptr| unsafe { node_ptr.as_ref() })
    }

    pub fn current_mut(&mut self) -> Option<&mut T> {
        self.current
            .map(|mut node_ptr| unsafe { node_ptr.as_mut() })
    }

    pub fn is_dangling(&self) -> bool {
        self.current.is_none()
    }

    pub fn move_next(&mut self) -> Option<&mut T> {
        let next = self.current().and_then(|node| node.next());

        self.current = next;

        self.current_mut()
    }

    pub fn move_prev(&mut self) -> Option<&mut T> {
        let prev = self.current().and_then(|node| node.prev());

        self.current = prev;

        self.current_mut()
    }

    pub fn remove_current(&mut self) -> Option<NonNull<T>> {
        let mut current_ptr = self.current.take()?;

        let list = unsafe { self.list.as_mut() };
        let current_node = unsafe { current_ptr.as_mut() };

        let prev = current_node.prev();
        let next = current_node.next();

        match prev {
            Some(mut prev_node) => unsafe { prev_node.as_mut().set_next(next) },
            None => list.head = next,
        }

        match next {
            Some(mut next_node) => unsafe { next_node.as_mut().set_prev(prev) },
            None => list.tail = prev,
        }

        list.len -= 1;

        current_node.set_next(None);
        current_node.set_prev(None);

        self.current = next;

        Some(current_ptr)
    }

    pub fn insert_before(&mut self, mut new_node: NonNull<T>) {
        assert_detached(new_node);

        let list = unsafe { self.list.as_mut() };

        match self.current {
            Some(mut current_node) => {
                let prev_node = unsafe { current_node.as_mut().prev() };

                unsafe {
                    new_node.as_mut().set_next(Some(current_node));
                    new_node.as_mut().set_prev(prev_node);
                    current_node.as_mut().set_prev(Some(new_node));
                }

                // update the previous neighbor
                match prev_node {
                    Some(mut p) => unsafe { p.as_mut().set_next(Some(new_node)) },
                    None => list.head = Some(new_node),
                }
                list.len += 1;
            }
            None => {
                list.push_back(new_node);
            }
        }
        self.current = Some(new_node);
    }

    pub fn insert_after(&mut self, mut new_node: NonNull<T>) {
        assert_detached(new_node);

        let list = unsafe { self.list.as_mut() };

        match self.current {
            Some(mut current_node) => {
                let next_node = unsafe { current_node.as_mut().next() };

                unsafe {
                    new_node.as_mut().set_next(next_node);
                    new_node.as_mut().set_prev(Some(current_node));
                    current_node.as_mut().set_next(Some(new_node));
                }

                match next_node {
                    Some(mut n) => unsafe { n.as_mut().set_prev(Some(new_node)) },
                    None => list.tail = Some(new_node),
                }
                list.len += 1;
            }
            None => {
                list.push_front(new_node);
            }
        }
        self.current = Some(new_node);
    }

    pub fn split_after(&mut self) -> IntrusiveList<T> {
        let Some(mut current_ptr) = self.current else {
            return IntrusiveList::new();
        };

        let Some(mut new_head_ptr) = (unsafe { current_ptr.as_ref().next() }) else {
            return IntrusiveList::new();
        };

        let list = unsafe { self.list.as_mut() };
        let old_tail = list.tail;

        unsafe {
            new_head_ptr.as_mut().set_prev(None);
            current_ptr.as_mut().set_next(None);
            list.tail = Some(current_ptr);
        }

        let mut moved_nodes_count = 0;
        let mut temp_node = Some(new_head_ptr);
        while let Some(node) = temp_node {
            moved_nodes_count += 1;
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

    pub fn splice_after(&mut self, other: &mut IntrusiveList<T>) {
        if other.is_empty() {
            return;
        }

        let mut other_head = other.head.take().unwrap();
        let mut other_tail = other.tail.take().unwrap();

        let other_len = other.len;
        other.len = 0;

        let list = unsafe { self.list.as_mut() };

        match self.current {
            Some(mut current_ptr) => {
                let original_next = unsafe { current_ptr.as_ref().next() };

                unsafe {
                    current_ptr.as_mut().set_next(Some(other_head));
                    other_head.as_mut().set_prev(Some(current_ptr));
                }

                match original_next {
                    Some(mut next_ptr) => unsafe {
                        other_tail.as_mut().set_next(Some(next_ptr));
                        next_ptr.as_mut().set_prev(Some(other_tail));
                    },
                    None => {
                        list.tail = Some(other_tail);
                    }
                }
            }
            None => match list.tail {
                Some(mut old_tail) => {
                    unsafe {
                        old_tail.as_mut().set_next(Some(other_head));
                        other_head.as_mut().set_prev(Some(old_tail));
                    }
                    list.tail = Some(other_tail);
                }
                None => {
                    list.head = Some(other_head);
                    list.tail = Some(other_tail);
                }
            },
        }
        list.len += other_len;
    }
}

#[inline]
fn assert_detached<T: Linkable>(node: NonNull<T>) {
    assert!(
        unsafe { node.as_ref().next().is_none() && node.as_ref().prev().is_none() },
        "Node is already in a list"
    );
}
