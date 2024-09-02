use std::fmt::Debug;
use std::ptr;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

struct Node<T> {
    next: AtomicPtr<Node<T>>,
    data: T,
}

#[derive(Debug)]
pub enum PoolError {
    Empty,
}

pub struct ConcurrentPool<T> {
    head: AtomicPtr<Node<T>>,
    tail: AtomicPtr<Node<T>>,
    size: AtomicUsize,
}

impl<T> ConcurrentPool<T> {
    pub fn new() -> Self {
        let sentinel_node = Box::into_raw(Box::new(Node {
            data: unsafe { std::mem::zeroed() },
            next: AtomicPtr::new(ptr::null_mut()),
        }));

        Self {
            head: AtomicPtr::new(sentinel_node),
            tail: AtomicPtr::new(sentinel_node),
            size: AtomicUsize::new(0),
        }
    }

    pub fn push(&self, val: T) -> Result<(), T> {
        let node = Box::into_raw(Box::new(Node {
            data: val,
            next: AtomicPtr::new(ptr::null_mut()),
        }));

        loop {
            let tail = self.tail.load(Ordering::Acquire);
            let next = unsafe { (*tail).next.load(Ordering::Acquire) };
            if next.is_null() {
                if unsafe {
                    (*tail)
                        .next
                        .compare_exchange(next, node, Ordering::Release, Ordering::Relaxed)
                }
                .is_ok()
                {
                    self.tail
                        .compare_exchange(tail, node, Ordering::Release, Ordering::Relaxed)
                        .ok();

                    self.size.fetch_add(1, Ordering::Relaxed);
                    return Ok(());
                }
            } else {
                self.tail
                    .compare_exchange(tail, next, Ordering::Release, Ordering::Relaxed)
                    .ok();
            }
        }
    }

    pub fn pop(&self) -> Result<T, PoolError> {
        loop {
            let head = self.head.load(Ordering::Acquire);
            let next = unsafe { (*head).next.load(Ordering::Acquire) };
            if next.is_null() {
                return Err(PoolError::Empty);
            }

            if self
                .head
                .compare_exchange(head, next, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                unsafe {
                    let data = ptr::read(&(*next).data);
                    std::mem::drop(Box::from_raw(head));
                    self.size.fetch_sub(1, Ordering::Relaxed);
                    return Ok(data);
                }
            }
        }
    }

    pub fn peek(&self) -> Option<&T> {
        loop {
            let head = self.head.load(Ordering::Acquire);
            let next = unsafe { (*head).next.load(Ordering::Acquire) };
            if next.is_null() {
                return None;
            }

            if head == self.head.load(Ordering::Relaxed) {
                return Some(unsafe { &(*head).data });
            }
        }
    }

    pub fn clear(&self) {
        while self.pop().is_ok() {}
    }

    pub fn is_empty(&self) -> bool {
        self.size.load(Ordering::Relaxed) == 0
    }

    pub fn len(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }

    pub fn try_pop(&self) -> Option<T> {
        self.pop().ok()
    }

    pub fn push_range<I>(&self, iter: I) -> usize
    where
        I: IntoIterator<Item = T>,
    {
        let mut count = 0;
        for item in iter {
            if self.push(item).is_ok() {
                count += 1;
            }
        }

        count
    }

    pub fn pop_range(&self, n: usize) -> Vec<T> {
        let mut res = Vec::with_capacity(n);
        for _ in 0..n {
            if let Ok(item) = self.pop() {
                res.push(item);
            } else {
                break;
            }
        }

        res
    }
}

pub struct Drain<'a, T> {
    pool: &'a ConcurrentPool<T>,
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.pool.pop().ok()
    }
}

pub struct Iter<'a, T> {
    curr: *const Node<T>,
    _marker: std::marker::PhantomData<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.is_null() {
            None
        } else {
            unsafe {
                let res = &(*self.curr).data;
                self.curr = (*self.curr).next.load(Ordering::Acquire);
                Some(res)
            }
        }
    }
}

impl<T> FromIterator<T> for ConcurrentPool<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let pool = ConcurrentPool::new();
        pool.push_range(iter);

        pool
    }
}

impl<T: Debug> Debug for ConcurrentPool<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConcurrentPool")
            .field("head", &self.head)
            .field("tail", &self.tail)
            .field("size", &self.size)
            .finish()
    }
}

impl<T> Drop for ConcurrentPool<T> {
    fn drop(&mut self) {
        self.clear();
        let head = self.head.load(Ordering::Relaxed);
        unsafe {
            drop(Box::from_raw(head));
        }
    }
}
