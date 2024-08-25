mod lockfree_stack;

use std::sync::Arc;
use std::thread;

use lockfree_stack::LockFreeStack;

fn main() {
    println!("Running single-threaded tests...");
    let stack = LockFreeStack::new();

    stack.push(1);
    stack.push(2);
    stack.push(3);
    assert_eq!(stack.try_pop(), Some(3));
    assert_eq!(stack.try_pop(), Some(2));

    stack.push(4);
    assert_eq!(stack.try_peek(), Some(&4));
    assert_eq!(stack.try_pop(), Some(4));

    assert!(!stack.is_empty());
    assert_eq!(stack.try_pop(), Some(1));
    assert!(stack.is_empty());

    stack.push_range(vec![5, 6, 7, 8]);
    assert_eq!(stack.try_pop(), Some(5));
    assert_eq!(stack.try_pop(), Some(6));

    let popped = stack.try_pop_range(3);
    assert_eq!(popped, vec![7, 8]);
    assert!(stack.is_empty());

    stack.push_range(vec![9, 10, 11]);
    let vec = stack.to_vec();
    assert_eq!(vec, vec![9, 10, 11]);

    let mut iter_vec = Vec::new();
    for &item in stack.iter() {
        iter_vec.push(item);
    }
    
    assert_eq!(iter_vec, vec![9, 10, 11]);

    stack.clear();
    assert!(stack.is_empty());

    println!("Running multi-threaded tests...");
    let stack = Arc::new(LockFreeStack::new());
    let num_threads = 10;
    let operations_per_thread = 1000;

    let mut handles = vec![];

    for _ in 0..num_threads {
        let stack_clone = Arc::clone(&stack);
        handles.push(thread::spawn(move || {
            for i in 0..operations_per_thread {
                if i % 2 == 0 {
                    stack_clone.push(i);
                } else {
                    stack_clone.try_pop();
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final stack size: {}", stack.to_vec().len());
    println!("Stack contents: {:?}", stack.to_vec());

    let stack = Arc::new(LockFreeStack::new());
    let mut handles = vec![];

    for i in 0..num_threads {
        let stack_clone = Arc::clone(&stack);
        handles.push(thread::spawn(move || {
            let range = (i * 100..(i + 1) * 100).collect();
            stack_clone.push_range(range);
        }));
    }

    for _ in 0..num_threads {
        let stack_clone = Arc::clone(&stack);
        handles.push(thread::spawn(move || {
            stack_clone.try_pop_range(50);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!(
        "Final stack size after range operations: {}",
        stack.to_vec().len()
    );
    println!(
        "Stack contents after range operations: {:?}",
        stack.to_vec()
    );

    println!("All tests completed successfully!");
}
