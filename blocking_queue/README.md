# BlockingQueue

A thread-safe, blocking queue implementation in Rust.

## Features

- Thread-safe operations using `Arc<Mutex<>>` and `Condvar`
- Blocking and non-blocking pop operations
- Efficient internal storage using `VecDeque`
- Comprehensive API for queue manipulation and inspection
- Implements `Clone`, `Debug`, `Default`, and `From<Vec<T>>` traits
- Guaranteed `Send` and `Sync` for `T: Send`
