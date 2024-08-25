use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::thread;
use std::time::Duration;

pub struct LockFreeStack<T> {
    head: AtomicPtr<Node<T>>,
}

struct Node<T> {
    value: T,
    next: AtomicPtr<Node<T>>,
}

impl<T> LockFreeStack<T> {
    pub fn new() -> Self {
        LockFreeStack {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    pub fn push(&self, value: T) {
        let new_node = Box::into_raw(Box::new(Node {
            value,
            next: AtomicPtr::new(ptr::null_mut()),
        }));

        let mut backoff = 1;
        loop {
            let curr_head = self.head.load(Ordering::Acquire);
            unsafe { (*new_node).next.store(curr_head, Ordering::Relaxed) };
            if self
                .head
                .compare_exchange(curr_head, new_node, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }

            thread::sleep(Duration::from_nanos(backoff));
            backoff = backoff.saturating_mul(2);
        }
    }

    pub fn push_range(&self, mut items: Vec<T>) {
        if items.is_empty() {
            return;
        }

        let mut new_head = Box::into_raw(Box::new(Node {
            value: items.pop().unwrap(),
            next: AtomicPtr::new(ptr::null_mut()),
        }));

        let tail = new_head;
        while let Some(item) = items.pop() {
            let new_node = Box::into_raw(Box::new(Node {
                value: item,
                next: AtomicPtr::new(ptr::null_mut()),
            }));

            unsafe { (*new_node).next.store(new_head, Ordering::Relaxed) };
            new_head = new_node;
        }

        let mut backoff = 1;
        loop {
            let curr_head = self.head.load(Ordering::Acquire);
            unsafe { (*tail).next.store(curr_head, Ordering::Relaxed) };
            if self
                .head
                .compare_exchange(curr_head, new_head, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }

            thread::sleep(Duration::from_nanos(backoff));
            backoff = backoff.saturating_mul(2);
        }
    }

    pub fn try_pop(&self) -> Option<T> {
        let mut backoff = 1;
        loop {
            let curr_head = self.head.load(Ordering::Acquire);
            if curr_head.is_null() {
                return None;
            }

            let next_node = unsafe { (*curr_head).next.load(Ordering::Acquire) };
            if self
                .head
                .compare_exchange(curr_head, next_node, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                let old_node = unsafe { Box::from_raw(curr_head) };
                return Some(old_node.value);
            }

            thread::sleep(Duration::from_nanos(backoff));
            backoff = backoff.saturating_mul(2);
        }
    }

    pub fn try_pop_range(&self, count: usize) -> Vec<T> {
        let mut result = Vec::with_capacity(count);
        let mut backoff = 1;
        loop {
            let curr_head = self.head.load(Ordering::Acquire);
            if curr_head.is_null() {
                break;
            }

            let mut next = curr_head;
            let mut nodes_count = 0;
            while nodes_count < count && !next.is_null() {
                next = unsafe { (*next).next.load(Ordering::Acquire) };
                nodes_count += 1;
            }

            if self
                .head
                .compare_exchange(curr_head, next, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                let mut current = curr_head;
                for _ in 0..nodes_count {
                    let node = unsafe { Box::from_raw(current) };
                    result.push(node.value);
                    current = node.next.load(Ordering::Relaxed);
                }
                break;
            }

            thread::sleep(Duration::from_nanos(backoff));
            backoff = backoff.saturating_mul(2);
        }
        result
    }

    pub fn try_peek(&self) -> Option<&T> {
        let curr_head = self.head.load(Ordering::Relaxed);
        if curr_head.is_null() {
            None
        } else {
            unsafe { Some(&(*curr_head).value) }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Relaxed).is_null()
    }

    pub fn clear(&self) {
        while self.try_pop().is_some() {}
    }

    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        let mut result = Vec::new();
        let mut curr_head = self.head.load(Ordering::Acquire);
        while !curr_head.is_null() {
            unsafe {
                result.push((*curr_head).value.clone());
                curr_head = (*curr_head).next.load(Ordering::Acquire);
            }
        }

        result
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            current: self.head.load(Ordering::Acquire),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> Drop for LockFreeStack<T> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T> IntoIterator for LockFreeStack<T>
where
    T: Clone,
{
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.to_vec().into_iter()
    }
}

pub struct Iter<'a, T> {
    current: *mut Node<T>,
    _marker: std::marker::PhantomData<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            None
        } else {
            unsafe {
                let node = &*self.current;
                self.current = node.next.load(Ordering::Relaxed);
                Some(&node.value)
            }
        }
    }
}
