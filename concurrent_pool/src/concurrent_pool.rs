use std::fmt::Debug;
use std::mem::MaybeUninit;
use std::ptr;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

struct Node<T> {
    next: AtomicPtr<Node<T>>,
    data: MaybeUninit<T>,
}

impl<T> Node<T> {
    fn new(value: T) -> *mut Node<T> {
        let boxed = Box::new(Self {
            next: AtomicPtr::new(ptr::null_mut()),
            data: MaybeUninit::new(value),
        });

        Box::into_raw(boxed)
    }

    fn new_sentinel() -> *mut Node<T> {
        let boxed = Box::new(Self {
            next: AtomicPtr::new(ptr::null_mut()),
            data: MaybeUninit::uninit(),
        });

        Box::into_raw(boxed)
    }
}

/// Error returned from `pop` when the pool is empty.
#[derive(Debug)]
pub enum PoolError {
    Empty,
}

/// A concurrent lock-free FIFO pool (Michael–Scott queue).
/// Note: size() is maintained as an atomic counter for quick queries,
/// but it's best-effort for concurrency (it is accurate but may be slightly
/// delayed relative to in-flight operations).
pub struct ConcurrentPool<T> {
    head: AtomicPtr<Node<T>>,
    tail: AtomicPtr<Node<T>>,
    size: AtomicUsize,
}

impl<T> ConcurrentPool<T> {
    /// Create a new empty pool.
    pub fn new() -> Self {
        let sentinel = Node::new_sentinel();
        Self {
            head: AtomicPtr::new(sentinel),
            tail: AtomicPtr::new(sentinel),
            size: AtomicUsize::new(0),
        }
    }

    /// Push a single element onto the queue.
    /// Returns `Ok(())` on success (always), or `Err(val)` if the operation somehow failed.
    /// In this design push can't fail, so we always return Ok.
    pub fn push(&self, val: T) -> Result<(), T> {
        let node = Node::new(val);

        loop {
            let tail = self.tail.load(Ordering::Acquire);
            let tail_next = unsafe { (*tail).next.load(Ordering::Acquire) };

            // If tail's next is null, try to link our node there
            if tail_next.is_null() {
                if unsafe {
                    // link node at tail->next
                    (*tail).next.compare_exchange(
                        ptr::null_mut(),
                        node,
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    )
                }
                .is_ok()
                {
                    // Advance tail to node (best-effort)
                    let _ = self.tail.compare_exchange(
                        tail,
                        node,
                        Ordering::Release,
                        Ordering::Relaxed,
                    );
                    self.size.fetch_add(1, Ordering::Release);
                    return Ok(());
                }
                // else someone linked concurrently; retry
            } else {
                // Tail not pointing to last node — try to advance it
                let _ = self.tail.compare_exchange(
                    tail,
                    tail_next,
                    Ordering::Release,
                    Ordering::Relaxed,
                );
            }
        }
    }

    /// Pop a single value from the queue. Returns Err(PoolError::Empty) if empty.
    pub fn pop(&self) -> Result<T, PoolError> {
        loop {
            let head = self.head.load(Ordering::Acquire);
            let tail = self.tail.load(Ordering::Acquire);
            let next = unsafe { (*head).next.load(Ordering::Acquire) };

            if next.is_null() {
                return Err(PoolError::Empty);
            }

            // If head == tail, and next != null, try to advance tail (helping)
            if head == tail {
                let _ =
                    self.tail
                        .compare_exchange(tail, next, Ordering::Release, Ordering::Relaxed);
                continue; // retry the main loop after helping
            }

            // Try to advance head to next
            if self
                .head
                .compare_exchange(head, next, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                // take data from next, drop old head
                let res = unsafe {
                    let value = ptr::read((*next).data.as_ptr());
                    // head was sentinel or older node; drop the boxed head
                    drop(Box::from_raw(head));
                    value
                };

                self.size.fetch_sub(1, Ordering::Release);
                return Ok(res);
            }
            // else CAS failed, retry
        }
    }

    /// Try pop returning Option<T>
    pub fn try_pop(&self) -> Option<T> {
        self.pop().ok()
    }

    /// Returns a cloned value from the head (without removing) if T: Clone.
    /// This is safe and does not return a reference that might be invalidated.
    pub fn peek_clone(&self) -> Option<T>
    where
        T: Clone,
    {
        let head = self.head.load(Ordering::Acquire);
        let next = unsafe { (*head).next.load(Ordering::Acquire) };
        if next.is_null() {
            None
        } else {
            Some(unsafe { (&*((*next).data.as_ptr())).clone() })
        }
    }

    /// Unsafe: return a reference to the head element (not removed).
    /// The caller must guarantee there are no concurrent `pop`/`clear` calls that
    /// would free the referenced node while the reference is used.
    ///
    /// Prefer `peek_clone` for concurrency safety.
    pub unsafe fn peek_ref(&self) -> Option<&T> {
        unsafe {
            let head = self.head.load(Ordering::Acquire);
            let next = (*head).next.load(Ordering::Acquire);
            if next.is_null() {
                None
            } else {
                Some(&*((*next).data.as_ptr()))
            }
        }
    }

    /// Clear the pool by popping everything.
    pub fn clear(&self) {
        while self.pop().is_ok() {}
    }

    /// Return whether empty
    pub fn is_empty(&self) -> bool {
        self.size.load(Ordering::Acquire) == 0
    }

    /// Return length (atomic)
    pub fn len(&self) -> usize {
        self.size.load(Ordering::Acquire)
    }

    /// Push a range, optimized: link nodes first and attach once to tail.
    /// Returns number of items pushed.
    pub fn push_range<I>(&self, iter: I) -> usize
    where
        I: IntoIterator<Item = T>,
    {
        let mut batch_head: *mut Node<T> = ptr::null_mut();
        let mut batch_tail: *mut Node<T> = ptr::null_mut();
        let mut count = 0usize;

        for item in iter {
            let node = Node::new(item);
            if batch_head.is_null() {
                batch_head = node;
            } else {
                unsafe {
                    (*batch_tail).next.store(node, Ordering::Relaxed);
                }
            }
            batch_tail = node;
            count += 1;
        }

        if batch_head.is_null() {
            return 0;
        }

        loop {
            let tail = self.tail.load(Ordering::Acquire);
            let tail_next = unsafe { (*tail).next.load(Ordering::Acquire) };

            if tail_next.is_null() {
                // try to attach entire batch
                if unsafe {
                    (*tail).next.compare_exchange(
                        ptr::null_mut(),
                        batch_head,
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    )
                }
                .is_ok()
                {
                    // advance tail to batch_tail (best-effort)
                    let _ = self.tail.compare_exchange(
                        tail,
                        batch_tail,
                        Ordering::Release,
                        Ordering::Relaxed,
                    );
                    self.size.fetch_add(count, Ordering::Release);
                    return count;
                }
                // failed, someone else inserted concurrently — retry
            } else {
                // advance tail
                let _ = self.tail.compare_exchange(
                    tail,
                    tail_next,
                    Ordering::Release,
                    Ordering::Relaxed,
                );
            }
        }
    }

    /// Pop up to `n` elements and return them in a Vec.
    pub fn pop_range(&self, n: usize) -> Vec<T> {
        let mut res = Vec::with_capacity(n);
        for _ in 0..n {
            if let Ok(v) = self.pop() {
                res.push(v);
            } else {
                break;
            }
        }
        res
    }

    /// Drain iterator consuming items via `pop`.
    pub const fn drain(&self) -> Drain<'_, T> {
        Drain { pool: self }
    }

    /// A non-concurrent iterator over current contents starting from head->next.
    /// WARNING: This iterator yields references into heap nodes that could be popped
    /// by concurrent threads. Use only when there are no concurrent mutators (or
    /// use external synchronization).
    pub fn iter(&self) -> Iter<'_, T> {
        let head = self.head.load(Ordering::Acquire);
        let first = unsafe { (*head).next.load(Ordering::Acquire) };
        Iter {
            curr: first,
            _marker: std::marker::PhantomData,
        }
    }
}

pub struct Drain<'a, T> {
    pool: &'a ConcurrentPool<T>,
}

impl<T> Iterator for Drain<'_, T> {
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
                let res = &*((*self.curr).data.as_ptr());
                self.curr = (*self.curr).next.load(Ordering::Acquire);
                Some(res)
            }
        }
    }
}

impl<T> FromIterator<T> for ConcurrentPool<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let pool = Self::new();
        pool.push_range(iter);
        pool
    }
}

impl<T: Debug> Debug for ConcurrentPool<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Small debug: don't try to print whole queue (could be large)
        f.debug_struct("ConcurrentPool")
            .field("head", &self.head.load(Ordering::Relaxed))
            .field("tail", &self.tail.load(Ordering::Relaxed))
            .field("size", &self.size.load(Ordering::Relaxed))
            .finish()
    }
}

impl<T> Drop for ConcurrentPool<T> {
    fn drop(&mut self) {
        while let Ok(_) = self.pop() {}
        let sentinel = self.head.load(Ordering::Relaxed);
        if !sentinel.is_null() {
            unsafe {
                // sentinel may have uninitialized data; ensure we don't try to drop data
                let _ = Box::from_raw(sentinel);
            }
        }
    }
}
