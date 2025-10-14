use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::Duration;

mod concurrent_pool;
use concurrent_pool::{ConcurrentPool, PoolError};

fn main() {
    println!("=== ConcurrentPool Demonstration ===");
    let pool: ConcurrentPool<i32> = ConcurrentPool::new();

    println!("\nPushing single elements...");
    for i in 0..5 {
        pool.push(i).unwrap();
        println!("  pushed {i}");
    }

    println!("len after pushes = {}", pool.len());
    if let Some(x) = pool.peek_clone() {
        println!("peek_clone() = {x}");
    }

    println!("\nPopping all elements...");
    while let Ok(v) = pool.pop() {
        println!("  popped {v}");
    }

    println!("is_empty() = {}", pool.is_empty());
    match pool.pop() {
        Ok(v) => println!("Unexpected: popped {v}"),
        Err(PoolError::Empty) => println!("pop() correctly returned PoolError::Empty"),
    }

    println!("\nPushing range of 10 elements...");
    let pushed = pool.push_range(10..20);
    println!("push_range() pushed {pushed}, len = {}", pool.len());

    println!("Pop 5 elements using pop_range...");
    let batch = pool.pop_range(5);
    println!("  popped = {batch:?}");
    println!("len now = {}", pool.len());

    println!("\nDraining rest via try_pop...");
    while let Some(v) = pool.try_pop() {
        println!("  try_pop() = {v}");
    }

    println!("is_empty() = {}", pool.is_empty());

    println!("\nTesting unsafe peek_ref...");
    pool.push_range(vec![111, 222, 333]);
    unsafe {
        if let Some(r) = pool.peek_ref() {
            println!("peek_ref() = {r}");
        }
    }

    println!("\nIterating over pool contents:");
    for v in pool.iter() {
        println!("  iter saw {v}");
    }

    println!("Draining using drain()...");
    for v in pool.drain() {
        println!("  drained {v}");
    }
    println!("len after drain = {}", pool.len());

    pool.push_range(1..=5);
    println!("\nlen before clear = {}", pool.len());
    pool.clear();
    println!("len after clear = {}", pool.len());

    println!("\nBuilding pool from iterator...");
    let new_pool: ConcurrentPool<i32> = (50..55).collect();
    println!("new_pool len = {}", new_pool.len());
    println!("Debug: {new_pool:?}");

    println!("\nIterating new_pool:");
    for x in new_pool.iter() {
        println!("  {x}");
    }

    println!("\n=== Concurrent stress demo ===");
    const PRODUCERS: usize = 4;
    const CONSUMERS: usize = 4;
    const ITEMS: usize = 1000;

    let pool_arc = Arc::new(ConcurrentPool::new());
    let barrier = Arc::new(Barrier::new(PRODUCERS + CONSUMERS));
    let collected = Arc::new(Mutex::new(Vec::new()));

    let mut threads = Vec::new();
    for p in 0..PRODUCERS {
        let pool = Arc::clone(&pool_arc);
        let barrier = Arc::clone(&barrier);
        threads.push(thread::spawn(move || {
            barrier.wait();
            for i in 0..ITEMS {
                pool.push(p * 1000 + i).unwrap();
                if i % 200 == 0 {
                    thread::sleep(Duration::from_millis(1));
                }
            }
        }));
    }

    for _ in 0..CONSUMERS {
        let pool = Arc::clone(&pool_arc);
        let barrier = Arc::clone(&barrier);
        let collected = Arc::clone(&collected);
        threads.push(thread::spawn(move || {
            barrier.wait();
            loop {
                if let Some(v) = pool.try_pop() {
                    collected.lock().unwrap().push(v);
                } else {
                    if pool.is_empty() {
                        break;
                    }
                    thread::yield_now();
                }
            }
        }));
    }

    for t in threads {
        t.join().unwrap();
    }

    let total = collected.lock().unwrap().len();
    println!(
        "Concurrent phase complete — popped {} elements (expected {}).",
        total,
        PRODUCERS * ITEMS
    );

    println!("Final len = {}", pool_arc.len());
    pool_arc.clear();
    println!("After clear: is_empty() = {}", pool_arc.is_empty());

    println!("\n=== Done ===");
}
