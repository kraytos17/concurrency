mod blocking_stack;

use blocking_stack::BlockingStack;
use std::{sync::Arc, thread, time::Duration};

fn main() {
    println!("Testing Single-threaded BlockingStack...");

    let stack = BlockingStack::new();
    println!("Created new stack: {:?}", stack);

    stack.push(1);
    stack.push(2);
    stack.push(3);
    println!("After pushing 1, 2, 3: {:?}", stack);

    assert_eq!(stack.pop(), 3);
    assert_eq!(stack.pop(), 2);
    println!("After popping twice: {:?}", stack);

    assert_eq!(stack.try_pop(), Some(1));
    assert_eq!(stack.try_pop(), None);
    println!("After try_pop: {:?}", stack);

    assert!(stack.is_empty());
    assert_eq!(stack.len(), 0);
    println!(
        "Stack is empty: {}, length: {}",
        stack.is_empty(),
        stack.len()
    );

    stack.push(4);
    assert_eq!(stack.peek(), Some(4));
    println!("Peeked: {:?}", stack.peek());

    stack.clear();
    assert!(stack.is_empty());
    println!("Cleared stack: {:?}", stack);

    stack.push(5);
    stack.push(6);
    stack.push(7);
    let drained = stack.drain();
    assert_eq!(drained, vec![7, 6, 5]);
    assert!(stack.is_empty());
    println!("Drained: {:?}, Stack now: {:?}", drained, stack);
    println!("Capacity: {}", stack.capacity());

    stack.push(8);
    stack.push(9);
    assert!(stack.contains(&8));
    assert!(!stack.contains(&10));
    println!(
        "Contains 8: {}, Contains 10: {}",
        stack.contains(&8),
        stack.contains(&10)
    );

    let reversed = stack.reverse();
    assert_eq!(reversed, vec![8, 9]);
    println!("Reversed: {:?}", reversed);

    let cloned_stack = stack.clone();
    assert_eq!(stack.peek(), cloned_stack.peek());
    println!(
        "Original stack: {:?}, Cloned stack: {:?}",
        stack, cloned_stack
    );

    println!("Testing Multi-threaded BlockingStack");
    let stack = Arc::new(BlockingStack::new());
    let stack_clone = Arc::clone(&stack);

    let producer = thread::spawn(move || {
        for i in 0..5 {
            stack_clone.push(i);
            println!("Produced: {}", i);
            thread::sleep(Duration::from_millis(100));
        }
    });

    let consumer = thread::spawn(move || {
        for _ in 0..5 {
            let item = stack.pop();
            println!("Consumed: {}", item);
            thread::sleep(Duration::from_millis(150));
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();

    println!("Multi-threaded test completed.");

    let stack = Arc::new(BlockingStack::new());
    let stack_clone = Arc::clone(&stack);

    let blocking_thread = thread::spawn(move || {
        println!("Waiting for item...");
        let item = stack_clone.pop();
        println!("Received item: {}", item);
    });

    thread::sleep(Duration::from_secs(1));
    println!("Pushing item to unblock thread");
    stack.push(42);

    blocking_thread.join().unwrap();
}
