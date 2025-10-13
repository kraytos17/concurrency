use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Condvar, Mutex, PoisonError};

/// A thread-safe blocking stack (LIFO) supporting concurrent push/pop operations.
///
/// If the stack is empty, `pop()` will block until an item becomes available.
/// Cloned instances share the same internal stack and condition variable.
#[derive(Default)]
pub struct BlockingStack<T> {
    inner: Arc<Inner<T>>,
}

#[derive(Default)]
struct Inner<T> {
    stack: Mutex<VecDeque<T>>,
    cvar: Condvar,
}

impl<T> BlockingStack<T> {
    /// Creates a new empty blocking stack.
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                stack: Mutex::new(VecDeque::new()),
                cvar: Condvar::new(),
            }),
        }
    }

    /// Creates a new blocking stack with a preallocated capacity.
    #[inline]
    pub fn _with_capacity(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Inner {
                stack: Mutex::new(VecDeque::with_capacity(capacity)),
                cvar: Condvar::new(),
            }),
        }
    }

    /// Pushes an item onto the stack and notifies one waiting thread.
    #[inline]
    pub fn push(&self, item: T) {
        let mut stack = self.inner.stack.lock().expect("mutex poisoned");
        stack.push_back(item);
        drop(stack);
        self.inner.cvar.notify_one();
    }

    /// Pops an item from the stack, blocking if it is empty.
    #[inline]
    pub fn pop(&self) -> T {
        let stack = self.inner.stack.lock().expect("mutex poisoned");
        let mut stack = self
            .inner
            .cvar
            .wait_while(stack, |s| s.is_empty())
            .expect("mutex poisoned");

        stack
            .pop_back()
            .expect("BlockingStack woke up but was empty")
    }

    /// Attempts to pop an item without blocking.
    #[inline]
    pub fn try_pop(&self) -> Option<T> {
        self.inner.stack.lock().ok()?.pop_back()
    }

    /// Returns `true` if the stack is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.stack.lock().map_or(true, |s| s.is_empty())
    }

    /// Returns the number of items currently in the stack.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.stack.lock().map_or(0, |s| s.len())
    }

    /// Returns a clone of the top element without removing it.
    #[inline]
    pub fn peek(&self) -> Option<T>
    where
        T: Clone,
    {
        self.inner.stack.lock().ok()?.back().cloned()
    }

    /// Clears all elements from the stack.
    #[inline]
    pub fn clear(&self) {
        if let Ok(mut s) = self.inner.stack.lock() {
            s.clear();
        }
    }

    /// Removes and returns all elements as a vector (from bottom to top).
    #[inline]
    pub fn drain(&self) -> Vec<T> {
        self.inner.stack.lock().map_or_else(
            |_| Vec::new(),
            |mut s| {
                let mut v = Vec::with_capacity(s.len());
                while let Some(item) = s.pop_back() {
                    v.push(item);
                }

                v
            },
        )
    }

    /// Returns the internal capacity of the stack.
    #[inline]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.inner.stack.lock().map_or(0, |s| s.capacity())
    }

    /// Checks whether the stack contains the specified item.
    #[inline]
    pub fn contains(&self, item: &T) -> bool
    where
        T: PartialEq,
    {
        self.inner.stack.lock().is_ok_and(|s| s.contains(item))
    }

    /// Returns a reversed clone of the internal data.
    #[inline]
    pub fn reversed(&self) -> VecDeque<T>
    where
        T: Clone,
    {
        let s = self.inner.stack.lock().expect("mutex poisoned");
        s.iter().rev().cloned().collect()
    }
}

impl<T> Clone for BlockingStack<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> fmt::Debug for BlockingStack<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stack = self
            .inner
            .stack
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        f.debug_struct("BlockingStack")
            .field("stack", &*stack)
            .finish()
    }
}

impl<T> From<Vec<T>> for BlockingStack<T> {
    fn from(vec: Vec<T>) -> Self {
        Self {
            inner: Arc::new(Inner {
                stack: Mutex::new(VecDeque::from(vec)),
                cvar: Condvar::new(),
            }),
        }
    }
}

unsafe impl<T: Send> Send for BlockingStack<T> {}
unsafe impl<T: Send> Sync for BlockingStack<T> {}
