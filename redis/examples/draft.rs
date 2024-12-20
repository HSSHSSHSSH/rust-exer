use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::collections::VecDeque;

struct SharedQueue<T> {
    queue: Mutex<VecDeque<T>>,
    not_empty: Condvar,
}

impl<T> SharedQueue<T> {
    fn new() -> Self {
        SharedQueue {
            queue: Mutex::new(VecDeque::new()),
            not_empty: Condvar::new(),
        }
    }

    fn push(&self, item: T) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(item);
        self.not_empty.notify_one();
    }

    fn pop(&self) -> T {
        let mut queue = self.queue.lock().unwrap();
        while queue.is_empty() {
            queue = self.not_empty.wait(queue).unwrap();
        }
        queue.pop_front().unwrap()
    }
}

fn producer_consumer_example() {
    let queue = Arc::new(SharedQueue::new());
    let queue2 = queue.clone();

    // 消费者
    let consumer = thread::spawn(move || {
        for _ in 0..5 {
            let item = queue2.pop();
            println!("消费者: 获取到数据 {}", item);
            thread::sleep(std::time::Duration::from_millis(500));
        }
    });

    // 生产者
    let producer = thread::spawn(move || {
        for i in 0..5 {
            println!("生产者: 放入数据 {}", i);
            queue.push(i);
            thread::sleep(std::time::Duration::from_millis(1000));
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}

fn main() {
    producer_consumer_example();
}   