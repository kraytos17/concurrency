use std::sync::atomic::{AtomicPtr, Ordering};

#[derive(Debug)]
pub struct LockFreeQueue<T> {
    head: AtomicPtr<Node<T>>,
    tail: AtomicPtr<Node<T>>,
}

#[derive(Debug)]
struct Node<T> {
    value: T,
    next: AtomicPtr<Node<T>>,
}

impl<T> LockFreeQueue<T> {
    pub fn new() -> Self {
        let sentinel_node = Box::into_raw(Box::new(Node {
            value: unsafe { std::mem::zeroed() },
            next: AtomicPtr::new(std::ptr::null_mut()),
        }));

        Self {
            head: AtomicPtr::new(sentinel_node),
            tail: AtomicPtr::new(sentinel_node),
        }
    }

    pub fn enqueue(&self, value: T) {
        let node = Box::into_raw(Box::new(Node {
            value,
            next: AtomicPtr::new(std::ptr::null_mut()),
        }));

        loop {
            let tail = self.tail.load(Ordering::Acquire);
            let next_node = unsafe { (*tail).next.load(Ordering::Acquire) };
            if tail == self.tail.load(Ordering::Relaxed) {
                if next_node.is_null() {
                    if unsafe {
                        (*tail)
                            .next
                            .compare_exchange(next_node, node, Ordering::Release, Ordering::Relaxed)
                            .is_ok()
                    } {
                        let _ = self.tail.compare_exchange(
                            tail,
                            node,
                            Ordering::Release,
                            Ordering::Relaxed,
                        );
                        return;
                    }
                } else {
                    let _ = self.tail.compare_exchange(
                        tail,
                        next_node,
                        Ordering::Release,
                        Ordering::Relaxed,
                    );
                }
            }
        }
    }

    pub fn dequeue(&self) -> Option<T> {
        loop {
            let head = self.head.load(Ordering::Acquire);
            let tail = self.tail.load(Ordering::Acquire);
            let next = unsafe { (*head).next.load(Ordering::Acquire) };
            if head == self.head.load(Ordering::Relaxed) {
                if head == tail && next.is_null() {
                    return None;
                }
                let _ =
                    self.tail
                        .compare_exchange(tail, next, Ordering::Release, Ordering::Relaxed);
            } else {
                let val = unsafe { std::ptr::read(&(*next).value) };
                if self
                    .head
                    .compare_exchange(head, next, Ordering::Release, Ordering::Relaxed)
                    .is_ok()
                {
                    unsafe {
                        drop(Box::from_raw(head));
                    }

                    return Some(val);
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        let head = self.head.load(Ordering::Acquire);
        let next = unsafe { (*head).next.load(Ordering::Acquire) };
        next.is_null()
    }

    pub fn peek(&self) -> Option<&T> {
        loop {
            let head = self.head.load(Ordering::Acquire);
            let next = unsafe { (*head).next.load(Ordering::Acquire) };
            if next.is_null() {
                return None;
            }
            if head == self.head.load(Ordering::Relaxed) {
                return Some(unsafe { &(*next).value });
            }
        }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            current: unsafe {
                (*self.head.load(Ordering::Acquire))
                    .next
                    .load(Ordering::Acquire)
            },
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> Drop for LockFreeQueue<T> {
    fn drop(&mut self) {
        while self.dequeue().is_some() {}
        let head = self.head.load(Ordering::Relaxed);
        unsafe {
            drop(Box::from_raw(head));
        }
    }
}

impl<T: Clone> Clone for LockFreeQueue<T> {
    fn clone(&self) -> Self {
        let new_queue = LockFreeQueue::new();
        let mut current = unsafe {
            (*(self.head.load(Ordering::Acquire)))
                .next
                .load(Ordering::Acquire)
        };

        while !current.is_null() {
            new_queue.enqueue(unsafe { (*current).value.clone() });
            current = unsafe { (*current).next.load(Ordering::Acquire) };
        }

        new_queue
    }
}

impl<T> Default for LockFreeQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Iter<'a, T> {
    current: *const Node<T>,
    _marker: std::marker::PhantomData<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            None
        } else {
            unsafe {
                let result = &(*self.current).value;
                self.current = (*self.current).next.load(Ordering::Acquire);
                Some(result)
            }
        }
    }
}

unsafe impl<T: Send> Send for LockFreeQueue<T> {}
unsafe impl<T: Sync> Sync for LockFreeQueue<T> {}
