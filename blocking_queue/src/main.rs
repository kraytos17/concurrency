mod blocking_queue;

use blocking_queue::BlockingQueue;
use std::{sync::Arc, thread, time::Duration};

fn main() {
    println!("Testing Single-threaded BlockingQueue...");

    let queue = BlockingQueue::new();
    println!("Created new queue: {:?}", queue);

    queue.push(1);
    queue.push(2);
    queue.push(3);
    println!("After pushing 1, 2, 3: {:?}", queue);

    assert_eq!(queue.pop(), 1);
    assert_eq!(queue.pop(), 2);
    println!("After popping twice: {:?}", queue);

    assert_eq!(queue.try_pop(), Some(3));
    assert_eq!(queue.try_pop(), None);
    println!("After try_pop: {:?}", queue);

    assert!(queue.is_empty());
    assert_eq!(queue.len(), 0);
    println!(
        "Queue is empty: {}, length: {}",
        queue.is_empty(),
        queue.len()
    );

    queue.push(4);
    assert_eq!(queue.peek(), Some(4));
    println!("Peeked: {:?}", queue.peek());

    queue.clear();
    assert!(queue.is_empty());
    println!("Cleared queue: {:?}", queue);

    queue.push(5);
    queue.push(6);
    queue.push(7);
    let drained = queue.drain();
    assert_eq!(drained, vec![5, 6, 7]);
    assert!(queue.is_empty());
    println!("Drained: {:?}, Queue now: {:?}", drained, queue);
    println!("Capacity: {}", queue.capacity());

    queue.push(8);
    queue.push(9);
    assert!(queue.contains(&8));
    assert!(!queue.contains(&10));
    println!(
        "Contains 8: {}, Contains 10: {}",
        queue.contains(&8),
        queue.contains(&10)
    );

    let reversed = queue.reverse();
    assert_eq!(reversed.clone().into_iter().collect::<Vec<_>>(), vec![9, 8]);
    println!("Reversed: {:?}", reversed);

    let cloned_queue = queue.clone();
    assert_eq!(queue.peek(), cloned_queue.peek());
    println!(
        "Original queue: {:?}, Cloned queue: {:?}",
        queue, cloned_queue
    );

    println!("Testing Multi-threaded BlockingQueue");
    let queue = Arc::new(BlockingQueue::new());
    let queue_clone = Arc::clone(&queue);

    let producer = thread::spawn(move || {
        for i in 0..5 {
            queue_clone.push(i);
            println!("Produced: {}", i);
            thread::sleep(Duration::from_millis(100));
        }
    });

    let consumer = thread::spawn(move || {
        for _ in 0..5 {
            let item = queue.pop();
            println!("Consumed: {}", item);
            thread::sleep(Duration::from_millis(150));
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();

    println!("Multi-threaded test completed.");

    let queue = Arc::new(BlockingQueue::new());
    let queue_clone = Arc::clone(&queue);

    let blocking_thread = thread::spawn(move || {
        println!("Waiting for item...");
        let item = queue_clone.pop();
        println!("Received item: {}", item);
    });

    thread::sleep(Duration::from_secs(1));
    println!("Pushing item to unblock thread");
    queue.push(42);

    blocking_thread.join().unwrap();
}
