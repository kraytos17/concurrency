# BlockingStack

## Overview

The `BlockingStack<T>` is a thread-safe, blocking stack implementation in Rust, utilizing synchronization primitives like `Arc`, `Mutex`, and `Condvar` to provide safe and efficient stack operations across multiple threads. This stack supports standard stack operations like push, pop, and peek, along with additional utility functions, making it a versatile and reliable data structure for concurrent applications.

## Features

- **Thread-Safe**: The `BlockingStack` ensures safe concurrent access by using `Mutex` to guard the stack and `Condvar` to block and wake threads as needed.
- **Blocking Pop Operation**: The `pop` method blocks the calling thread if the stack is empty, only returning when an item becomes available.
- **Non-blocking Try-Pop**: The `try_pop` method allows for a non-blocking attempt to pop an item, returning `None` if the stack is empty.
- **Peek and Contains**: Provides methods to peek at the top item and check if an item exists within the stack.
- **Capacity and Length Queries**: The stack allows querying its current length and capacity.
- **Reversal and Drain**: Supports reversing the stack and draining its contents into a vector.

