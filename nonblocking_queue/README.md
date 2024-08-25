# LockFreeQueue

A high-performance, lock-free queue implementation in Rust.

## Features

- Lock-free operations for high concurrency
- Thread-safe enqueue and dequeue operations
- Efficient memory management with proper cleanup
- Iterator support for easy traversal
- Implements `Clone`, `Debug`, `Default` traits
- Guaranteed `Send` and `Sync` for `T: Send`
