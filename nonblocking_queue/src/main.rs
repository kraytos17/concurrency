mod lockfree_queue;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use lockfree_queue::LockFreeQueue;

fn main() {
    let queue = LockFreeQueue::new();
    queue.enqueue(1);
    queue.enqueue(2);
    queue.enqueue(3);
    assert_eq!(queue.dequeue(), Some(1));
    assert_eq!(queue.dequeue(), Some(2));
    assert_eq!(queue.dequeue(), Some(3));
    assert_eq!(queue.dequeue(), None);

    assert!(queue.is_empty());
    queue.enqueue(4);
    assert!(!queue.is_empty());

    assert_eq!(queue.peek(), Some(&4));
    assert_eq!(queue.dequeue(), Some(4));
    assert_eq!(queue.peek(), None);

    queue.enqueue(5);
    queue.enqueue(6);
    queue.enqueue(7);
    let mut iter = queue.iter();
    assert_eq!(iter.next(), Some(&5));
    assert_eq!(iter.next(), Some(&6));
    assert_eq!(iter.next(), Some(&7));
    assert_eq!(iter.next(), None);

    let queue2 = queue.clone();
    assert_eq!(queue.dequeue(), Some(5));
    assert_eq!(queue2.dequeue(), Some(5));

    let queue = Arc::new(LockFreeQueue::new());
    let mut handles = vec![];

    for i in 0..10 {
        let q = Arc::clone(&queue);
        handles.push(thread::spawn(move || {
            for j in 0..1000 {
                q.enqueue(i * 1000 + j);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let mut dequeue_handles = vec![];
    for _ in 0..10 {
        let q = Arc::clone(&queue);
        dequeue_handles.push(thread::spawn(move || {
            let mut sum = 0;
            for _ in 0..1000 {
                if let Some(val) = q.dequeue() {
                    sum += val;
                }
            }
            sum
        }));
    }

    let mut total_sum = 0;
    for handle in dequeue_handles {
        total_sum += handle.join().unwrap();
    }

    assert_eq!(total_sum, 49995000);

    let queue = Arc::new(LockFreeQueue::new());
    let q_clone = Arc::clone(&queue);

    let producer = thread::spawn(move || {
        for i in 0..10000 {
            q_clone.enqueue(i);
            thread::sleep(Duration::from_nanos(1));
        }
    });

    let consumer = thread::spawn(move || {
        let mut sum = 0;
        for _ in 0..10000 {
            while queue.dequeue().is_none() {
                thread::yield_now();
            }
            if let Some(val) = queue.dequeue() {
                sum += val;
            }
        }
        sum
    });

    producer.join().unwrap();
    let sum = consumer.join().unwrap();
    assert_eq!(sum, 49995000);

    println!("All tests passed successfully!");
}
