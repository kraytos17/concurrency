use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Condvar, Mutex};

pub struct BlockingStack<T> {
    stack: Arc<Mutex<VecDeque<T>>>,
    cvar: Condvar,
}

impl<T> BlockingStack<T> {
    pub fn new() -> Self {
        Self {
            stack: Arc::new(Mutex::new(VecDeque::new())),
            cvar: Condvar::new(),
        }
    }

    pub fn _with_capacity(capacity: usize) -> Self {
        Self {
            stack: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            cvar: Condvar::new(),
        }
    }

    pub fn push(&self, item: T) {
        let mut stack = self.stack.lock().unwrap();
        stack.push_back(item);
        self.cvar.notify_one();
    }

    pub fn pop(&self) -> T {
        let mut stack = self.stack.lock().unwrap();
        while stack.is_empty() {
            stack = self.cvar.wait(stack).unwrap();
        }
        stack.pop_back().unwrap()
    }

    pub fn try_pop(&self) -> Option<T> {
        self.stack.lock().unwrap().pop_back()
    }

    pub fn is_empty(&self) -> bool {
        self.stack.lock().unwrap().is_empty()
    }

    pub fn len(&self) -> usize {
        self.stack.lock().unwrap().len()
    }

    pub fn peek(&self) -> Option<T>
    where
        T: Clone,
    {
        self.stack.lock().unwrap().back().cloned()
    }

    pub fn clear(&self) {
        self.stack.lock().unwrap().clear();
    }

    pub fn drain(&self) -> Vec<T> {
        self.stack.lock().unwrap().drain(..).collect()
    }

    pub fn capacity(&self) -> usize {
        self.stack.lock().unwrap().capacity()
    }

    pub fn contains(&self, item: &T) -> bool
    where
        T: PartialEq,
    {
        self.stack.lock().unwrap().contains(item)
    }

    pub fn reverse(&self) -> VecDeque<T>
    where
        T: Clone,
    {
        let mut stack = self.stack.lock().unwrap().clone();
        stack.make_contiguous().reverse();
        
        stack
    }
}

impl<T> Default for BlockingStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for BlockingStack<T> {
    fn clone(&self) -> Self {
        Self {
            stack: Arc::clone(&self.stack),
            cvar: Condvar::new(),
        }
    }
}

impl<T> fmt::Debug for BlockingStack<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stack = self.stack.lock().unwrap();
        f.debug_struct("BlockingStack")
            .field("stack", &*stack)
            .finish()
    }
}

impl<T> From<Vec<T>> for BlockingStack<T> {
    fn from(v: Vec<T>) -> Self {
        Self {
            stack: Arc::new(Mutex::new(VecDeque::from(v))),
            cvar: Condvar::new(),
        }
    }
}

unsafe impl<T: Send> Send for BlockingStack<T> {}
unsafe impl<T: Send> Sync for BlockingStack<T> {}
