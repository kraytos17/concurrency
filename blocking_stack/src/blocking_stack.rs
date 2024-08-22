use std::fmt;
use std::sync::{Arc, Condvar, Mutex};

pub struct BlockingStack<T> {
    stack: Arc<(Mutex<Vec<T>>, Condvar)>,
}

impl<T> BlockingStack<T> {
    pub fn new() -> Self {
        Self {
            stack: Arc::new((Mutex::new(Vec::new()), Condvar::new())),
        }
    }

    pub fn push(&self, item: T) {
        let (lock, cvar) = &*self.stack;
        let mut stack = lock.lock().unwrap();
        stack.push(item);
        cvar.notify_one();
    }

    pub fn pop(&self) -> T {
        let (lock, cvar) = &*self.stack;
        let mut stack = lock.lock().unwrap();
        while stack.is_empty() {
            stack = cvar.wait(stack).unwrap();
        }
        stack.pop().unwrap()
    }

    pub fn try_pop(&self) -> Option<T> {
        let (lock, _) = &*self.stack;
        let mut stack = lock.lock().unwrap();
        stack.pop()
    }

    pub fn is_empty(&self) -> bool {
        let (lock, _) = &*self.stack;
        let stack = lock.lock().unwrap();
        stack.is_empty()
    }

    pub fn len(&self) -> usize {
        let (lock, _) = &*self.stack;
        let stack = lock.lock().unwrap();
        stack.len()
    }

    pub fn peek(&self) -> Option<T>
    where
        T: Clone,
    {
        let (lock, _) = &*self.stack;
        let stack = lock.lock().unwrap();
        stack.last().cloned()
    }

    pub fn clear(&self) {
        let (lock, _) = &*self.stack;
        let mut stack = lock.lock().unwrap();
        stack.clear();
    }

    pub fn drain(&self) -> Vec<T> {
        let (lock, _) = &*self.stack;
        let mut stack = lock.lock().unwrap();
        stack.drain(..).collect()
    }

    pub fn capacity(&self) -> usize {
        let (lock, _) = &*self.stack;
        let stack = lock.lock().unwrap();
        stack.capacity()
    }

    pub fn contains(&self, item: &T) -> bool
    where
        T: PartialEq,
    {
        let (lock, _) = &*self.stack;
        let stack = lock.lock().unwrap();
        stack.contains(item)
    }

    pub fn reverse(&self) -> Vec<T>
    where
        T: Clone,
    {
        let (lock, _) = &*self.stack;
        let stack = lock.lock().unwrap();
        stack.iter().cloned().rev().collect()
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
        }
    }
}

impl<T> fmt::Debug for BlockingStack<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (lock, _) = &*self.stack;
        let stack = lock.lock().unwrap();
        f.debug_struct("BlockingStack")
            .field("stack", &stack)
            .finish()
    }
}
