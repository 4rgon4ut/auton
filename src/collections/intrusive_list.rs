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
    phantom: PhantomData<T>,
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

    pub fn head(&self) -> Option<NonNull<T>> {
        self.head
    }

    pub fn tail(&self) -> Option<NonNull<T>> {
        self.tail
    }

    pub fn push_front(&mut self, mut node: NonNull<T>) {
        let node_ref = unsafe { node.as_mut() };

        assert!(
            node_ref.next().is_none() && node_ref.prev().is_none(),
            "Node is already in a list"
        );

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
        let node_ref = unsafe { node.as_mut() };

        assert!(
            node_ref.next().is_none() && node_ref.prev().is_none(),
            "Node is already in a list"
        );

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
}

impl<T: Linkable> Default for IntrusiveList<T> {
    fn default() -> Self {
        Self::new()
    }
}

// TODO:
pub struct CursorMut<'a, T: Linkable> {
    list: NonNull<IntrusiveList<T>>,
    node: Option<NonNull<T>>,
    phantom: PhantomData<&'a mut T>,
}
