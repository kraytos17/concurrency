mod concurrent_pool;

use std::thread;
use std::time::Duration;

use concurrent_pool::{ConcurrentPool, PoolError};

fn main() {
    let pool: ConcurrentPool<i32> = ConcurrentPool::new();
    println!("Pushing elements to the pool");
    for i in 0..5 {
        pool.push(i).unwrap();
        println!("Pushed: {}", i);
    }

    println!("Pool size: {}", pool.len());

    if let Some(peeked) = pool.peek() {
        println!("Peeked element: {}", peeked);
    }

    println!("Popping elements from the pool");
    while let Ok(item) = pool.pop() {
        println!("Popped: {}", item);
    }

    println!("Is pool empty? {}", pool.is_empty());

    match pool.pop() {
        Ok(item) => println!("Popped: {}", item),
        Err(PoolError::Empty) => println!("Pool is empty"),
    }

    let range: Vec<i32> = (0..10).collect();
    let pushed = pool.push_range(range);
    println!("Pushed {} elements", pushed);
    let popped = pool.pop_range(5);
    println!("Popped range: {:?}", popped);
    let pool_arc = std::sync::Arc::new(pool);
    let push_thread = {
        let pool = pool_arc.clone();
        thread::spawn(move || {
            for i in 100..110 {
                pool.push(i).unwrap();
                thread::sleep(Duration::from_millis(10));
            }
        })
    };

    let pop_thread = {
        let pool = pool_arc.clone();
        thread::spawn(move || {
            for _ in 0..15 {
                if let Some(item) = pool.try_pop() {
                    println!("Popped in concurrent thread: {}", item);
                }
                thread::sleep(Duration::from_millis(15));
            }
        })
    };

    push_thread.join().unwrap();
    pop_thread.join().unwrap();

    println!("Final pool size: {}", pool_arc.len());
    println!("Final pool contents:");
    while let Ok(item) = pool_arc.pop() {
        println!("{}", item);
    }

    pool_arc.clear();
    println!("Pool cleared. Is empty? {}", pool_arc.is_empty());

    let vec = vec![1, 2, 3, 4, 5];
    let new_pool: ConcurrentPool<i32> = vec.into_iter().collect();
    println!("New pool from iterator, size: {}", new_pool.len());
    println!("Debug output of pool: {:?}", new_pool);
}
