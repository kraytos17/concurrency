use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Condvar, Mutex, PoisonError};

/// A thread-safe blocking queue that allows pushing and popping elements with blocking semantics.
///
/// Multiple producers and consumers can safely operate concurrently.
#[derive(Default)]
pub struct BlockingQueue<T> {
    inner: Arc<Inner<T>>,
}

#[derive(Default)]
struct Inner<T> {
    queue: Mutex<VecDeque<T>>,
    cvar: Condvar,
}

impl<T> BlockingQueue<T> {
    /// Creates a new empty blocking queue.
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                queue: Mutex::new(VecDeque::new()),
                cvar: Condvar::new(),
            }),
        }
    }

    /// Creates a new blocking queue with preallocated capacity.
    #[inline]
    pub fn _with_capacity(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Inner {
                queue: Mutex::new(VecDeque::with_capacity(capacity)),
                cvar: Condvar::new(),
            }),
        }
    }

    /// Pushes an item to the back of the queue and notifies one waiting thread.
    #[inline]
    pub fn push(&self, item: T) {
        let mut queue = self.inner.queue.lock().expect("mutex poisoned");
        queue.push_back(item);
        drop(queue);
        self.inner.cvar.notify_one();
    }

    /// Pops an item from the front of the queue, blocking if empty.
    #[inline]
    pub fn pop(&self) -> T
    where
        T: Clone,
    {
        let queue = self.inner.queue.lock().expect("mutex poisoned");
        let mut queue = self
            .inner
            .cvar
            .wait_while(queue, |q| q.is_empty())
            .expect("mutex poisoned");

        queue
            .front()
            .cloned()
            .unwrap_or_else(|| panic!("BlockingQueue.pop() woke up but queue was empty"));
        queue.pop_front().unwrap()
    }

    /// Attempts to pop an item from the queue without blocking.
    #[inline]
    pub fn try_pop(&self) -> Option<T> {
        self.inner.queue.lock().ok()?.pop_front()
    }

    /// Checks whether the queue is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.queue.lock().map_or(true, |q| q.is_empty())
    }

    /// Returns the number of elements currently in the queue.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.queue.lock().map_or(0, |q| q.len())
    }

    /// Returns a clone of the front element without removing it.
    #[inline]
    pub fn peek(&self) -> Option<T>
    where
        T: Clone,
    {
        self.inner.queue.lock().ok()?.front().cloned()
    }

    /// Clears all elements from the queue.
    #[inline]
    pub fn clear(&self) {
        if let Ok(mut q) = self.inner.queue.lock() {
            q.clear();
        }
    }

    /// Removes and returns all elements as a vector.
    #[inline]
    pub fn drain(&self) -> Vec<T> {
        self.inner
            .queue
            .lock()
            .map(|mut q| q.drain(..).collect())
            .unwrap_or_default()
    }

    /// Returns the capacity of the internal [`VecDeque`].
    #[inline]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.inner.queue.lock().map_or(0, |q| q.capacity())
    }

    /// Checks whether the queue contains the given element.
    #[inline]
    pub fn contains(&self, item: &T) -> bool
    where
        T: PartialEq,
    {
        self.inner.queue.lock().is_ok_and(|q| q.contains(item))
    }

    /// Returns a reversed clone of the queue contents.
    #[inline]
    pub fn reversed(&self) -> VecDeque<T>
    where
        T: Clone,
    {
        let mut queue = self.inner.queue.lock().expect("mutex poisoned").clone();
        queue.make_contiguous().reverse();

        queue
    }
}

impl<T> Clone for BlockingQueue<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> fmt::Debug for BlockingQueue<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let queue = self
            .inner
            .queue
            .lock()
            .unwrap_or_else(PoisonError::into_inner);

        f.debug_struct("BlockingQueue")
            .field("queue", &*queue)
            .finish()
    }
}

impl<T> From<Vec<T>> for BlockingQueue<T> {
    fn from(vec: Vec<T>) -> Self {
        Self {
            inner: Arc::new(Inner {
                queue: Mutex::new(VecDeque::from(vec)),
                cvar: Condvar::new(),
            }),
        }
    }
}

unsafe impl<T: Send> Send for BlockingQueue<T> {}
unsafe impl<T: Send> Sync for BlockingQueue<T> {}
