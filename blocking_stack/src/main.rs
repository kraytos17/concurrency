use blocking_stack::BlockingStack;
use std::{sync::Arc, thread};

mod blocking_stack;

fn main() {
    let stack = Arc::new(BlockingStack::new());
    let mut handles = vec![];

    for i in 0..10 {
        let stack = Arc::clone(&stack);
        let handle = thread::spawn(move || {
            stack.push(i);
            println!("Pushed {}, Stack capacity: {}", i, stack.capacity());
        });
        handles.push(handle);
    }

    for _ in 0..10 {
        let stack = Arc::clone(&stack);
        let handle = thread::spawn(move || {
            let item = stack.pop();
            println!("Popped {}", item);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final stack length: {}", stack.len());
    println!("Final stack is empty: {}", stack.is_empty());

    stack.push(100);
    let peeked = stack.peek();
    println!("Peeked item: {:?}", peeked);

    stack.push(200);
    stack.clear();
    println!("Stack length after clear: {}", stack.len());
    println!("Stack capacity: {}", stack.capacity());

    stack.push(400);
    let contains = stack.contains(&400);
    println!("Stack contains 400: {}", contains);
    let contains = stack.contains(&500);
    println!("Stack contains 500: {}", contains);

    stack.push(500);
    stack.push(600);
    let reversed_items = stack.reverse();
    println!("Reversed items in stack: {:?}", reversed_items);
}
