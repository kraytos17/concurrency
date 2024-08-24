use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Condvar, Mutex};

pub struct BlockingQueue<T> {
    queue: Arc<Mutex<VecDeque<T>>>,
    cvar: Condvar,
}

impl<T> BlockingQueue<T> {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            cvar: Condvar::new(),
        }
    }

    pub fn _with_capacity(capacity: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            cvar: Condvar::new(),
        }
    }

    pub fn push(&self, item: T) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(item);
        self.cvar.notify_one();
    }

    pub fn pop(&self) -> T {
        let mut queue = self.queue.lock().unwrap();
        while queue.is_empty() {
            queue = self.cvar.wait(queue).unwrap();
        }
        queue.pop_front().unwrap()
    }

    pub fn try_pop(&self) -> Option<T> {
        self.queue.lock().unwrap().pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.lock().unwrap().is_empty()
    }

    pub fn len(&self) -> usize {
        self.queue.lock().unwrap().len()
    }

    pub fn peek(&self) -> Option<T>
    where
        T: Clone,
    {
        self.queue.lock().unwrap().front().cloned()
    }

    pub fn clear(&self) {
        self.queue.lock().unwrap().clear();
    }

    pub fn drain(&self) -> Vec<T> {
        self.queue.lock().unwrap().drain(..).collect()
    }

    pub fn capacity(&self) -> usize {
        self.queue.lock().unwrap().capacity()
    }

    pub fn contains(&self, item: &T) -> bool
    where
        T: PartialEq,
    {
        self.queue.lock().unwrap().contains(item)
    }

    pub fn reverse(&self) -> VecDeque<T>
    where
        T: Clone,
    {
        let mut queue = self.queue.lock().unwrap().clone();
        queue.make_contiguous().reverse();

        queue
    }
}

impl<T> Default for BlockingQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for BlockingQueue<T> {
    fn clone(&self) -> Self {
        Self {
            queue: Arc::clone(&self.queue),
            cvar: Condvar::new(),
        }
    }
}

impl<T> fmt::Debug for BlockingQueue<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let queue = self.queue.lock().unwrap();
        f.debug_struct("BlockingQueue")
            .field("queue", &*queue)
            .finish()
    }
}

impl<T> From<Vec<T>> for BlockingQueue<T> {
    fn from(v: Vec<T>) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::from(v))),
            cvar: Condvar::new(),
        }
    }
}

unsafe impl<T: Send> Send for BlockingQueue<T> {}
unsafe impl<T: Send> Sync for BlockingQueue<T> {}
