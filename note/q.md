## 模块化

## 特征

## cell 内存图
```rs
fn memory_explanation() {
    // "asdf" 在只读数据段，地址假设为 0x1000
    // "qwer" 在只读数据段，地址假设为 0x2000
    
    let c = Cell::new("asdf");  // c 内部存储指向 0x1000 的指针
    let one = c.get();          // one 获得指向 0x1000 的指针的拷贝
    c.set("qwer");             // c 内部的指针更新为指向 0x2000
    let two = c.get();         // two 获得指向 0x2000 的指针的拷贝
}
```
## RefCell 借用判断
```rs
use std::cell::RefCell;

fn explain_borrow_counts() {
    let data = RefCell::new(vec![1, 2, 3]);
    
    // 1. 检查当前借用状态
    println!("初始状态:");
    println!("可变借用是否可用: {}", data.try_borrow_mut().is_ok());   // true
    println!("不可变借用是否可用: {}", data.try_borrow().is_ok());     // true
    
    // 2. 创建不可变借用
    let ref1 = data.borrow();
    let ref2 = data.borrow();
    println!("\n两个不可变借用后:");
    println!("可变借用是否可用: {}", data.try_borrow_mut().is_ok());   // false，因为有不可变借用
    println!("不可变借用是否可用: {}", data.try_borrow().is_ok());     // true，可以继续不可变借用
    
    // 释放不可变借用
    drop(ref1);
    drop(ref2);
    
    // 3. 创建可变借用
    {
        let mut mut_ref = data.borrow_mut();
        println!("\n可变借用存在时:");
        println!("可变借用是否可用: {}", data.try_borrow_mut().is_ok());   // false，已经有一个可变借用
        println!("不可变借用是否可用: {}", data.try_borrow().is_ok());     // false，有可变借用时不能不可变借用
        
        // 修改数据
        mut_ref.push(4);
    } // mut_ref 在这里被 drop
    
    // 4. 所有借用都释放后
    println!("\n所有借用释放后:");
    println!("可变借用是否可用: {}", data.try_borrow_mut().is_ok());   // true
    println!("不可变借用是否可用: {}", data.try_borrow().is_ok());     // true
    println!("最终数据: {:?}", data.borrow());
}

fn main() {
    explain_borrow_counts();
}


```

## NLL 中的"最后一次使用"是针对借用（引用）的，而不是变量本身

## 线程屏障 Barrier 使用中的问题

让我解释一下当 `Barrier` 的参数与实际等待的线程数不匹配时会发生什么：

1. **参数小于线程数（比如示例中 Barrier::new(5) 但创建了 6 个线程）**：
```rust:redis/examples/draft.rs
let barrier = Arc::new(Barrier::new(5));  // 只需要5个线程同步

for _ in 0..6 {  // 但创建了6个线程
    let b = barrier.clone();
    handles.push(thread::spawn(move|| {
        println!("before wait");
        b.wait();        // 当第5个线程到达时，前5个线程会被释放
                        // 第6个线程将永远等待，因为永远不会再有足够的线程到达
        println!("after wait");
    }));
}
```
结果：
- 前5个到达的线程会被正常释放
- 第6个线程会永远阻塞
- 程序不会正常结束（死锁）

2. **参数大于线程数（比如 Barrier::new(7) 但只有 6 个线程）**：
```rust:redis/examples/draft.rs
let barrier = Arc::new(Barrier::new(7));  // 需要7个线程同步

for _ in 0..6 {  // 但只创建了6个线程
    let b = barrier.clone();
    handles.push(thread::spawn(move|| {
        println!("before wait");
        b.wait();        // 所有6个线程都会在这里永远等待
                        // 因为永远不会有第7个线程到达
        println!("after wait");
    }));
}
```
结果：
- 所有线程都会永远阻塞
- 程序不会正常结束（死锁）

最佳实践：
- 确保 `Barrier` 的参数严格等于需要同步的线程数
- 如果线程数是动态的，考虑使用其他同步原语（如 `Mutex` 或 `Condvar`）
- 在开发时可以添加日志或打印语句来追踪到达 barrier 的线程数量

## 多线程中使用 Condvar 的注意事项
 condvar 的 wait 方法会接收一个 MutexGuard<'a, T>，且它会自动地暂时释放这个锁，使其他线程可以拿到锁并进行数据更新。
 同时当前线程在此处会被阻塞，直到被其他地方 notify 后，它会将原本的 MutexGuard<'a, T> 还给我们，即重新获取到了锁，同时唤醒了此线程。
 
 
