# LockFreeStack

`LockFreeStack<T>` is a lock-free, thread-safe stack data structure implemented in Rust. This stack is designed for concurrent environments, where multiple threads may push and pop elements simultaneously without the need for traditional locking mechanisms. The stack leverages atomic operations to ensure consistency and avoid race conditions.

## Features

- **Lock-free push and pop operations**: Enables safe concurrent access to the stack without the overhead of locks.
- **Support for bulk operations**: `push_range` and `try_pop_range` allow batch processing of elements.
- **Peek and check if empty**: Check the top of the stack or whether the stack is empty without removing elements.
- **Iterator support**: Traverse the stack with an iterator for easy element access.
- **Safe memory management**: The stack handles memory using Rust's ownership model, automatically cleaning up when dropped.
