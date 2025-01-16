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
 
 
## while let
```rs
// 基本形式
while let 模式 = 表达式 {
    // 代码块
}

// 等价的 loop + match 形式
loop {
    match 表达式 {
        模式 => { /* 代码块 */ },
        _ => break,
    }
}
```

## if let
```rs
// 基本形式
if let 模式 = 表达式 {
    // 代码块
}

// 等价的 match 形式
match 表达式 {
    模式 => { /* 代码块 */ },
    _ => (), // 不匹配时什么都不做
}
// 如果有 else 分支的基本形式
if let 模式 = 表达式 {
    // 代码块 1
} else {
    // 代码块 2
}

// 带 else 的等价 match 形式
match 表达式 {
    模式 => { /* 代码块 1 */ },
    _ => { /* 代码块 2 */ },
}
```

## Cursor




`cursor.read(&mut byte)` 是将 cursor 中的内容读取到 byte 缓冲区中。让我用例子详细说明这个过程：

```rust
use std::io::{Cursor, Read};

fn main() -> std::io::Result<()> {
    // 创建一个 cursor，内容是 [1, 2, 3, 4, 5]
    let mut cursor = Cursor::new(vec![1, 2, 3, 4, 5]);
    
    // 创建一个缓冲区 byte
    let mut byte = [0; 2];  // 长度为2的数组，初始值都是0
    
    // 1. 第一次读取
    println!("读取前 byte: {:?}", byte);        // [0, 0]
    let n1 = cursor.read(&mut byte)?;
    println!("读取后 byte: {:?}", byte);        // [1, 2]
    println!("读取的字节数: {}", n1);           // 2
    println!("cursor位置: {}", cursor.position()); // 2
    
    // 2. 第二次读取
    let n2 = cursor.read(&mut byte)?;
    println!("第二次读取 byte: {:?}", byte);     // [3, 4]
    println!("读取的字节数: {}", n2);           // 2
    println!("cursor位置: {}", cursor.position()); // 4
    
    // 3. 第三次读取（只剩一个字节了）
    let n3 = cursor.read(&mut byte)?;
    println!("第三次读取 byte: {:?}", byte);     // [5, 4] 注意第二个位置没变
    println!("读取的字节数: {}", n3);           // 1
    println!("cursor位置: {}", cursor.position()); // 5
    
    Ok(())
}
```


让我们看一些不同大小的缓冲区示例：

```rust
use std::io::{Cursor, Read};

fn main() -> std::io::Result<()> {
    let mut cursor = Cursor::new(vec![1, 2, 3, 4, 5]);
    
    // 1. 单字节读取
    let mut one_byte = [0; 1];
    cursor.read(&mut one_byte)?;
    println!("读取一个字节: {:?}", one_byte);  // [1]
    
    // 2. 两字节读取
    let mut two_bytes = [0; 2];
    cursor.read(&mut two_bytes)?;
    println!("读取两个字节: {:?}", two_bytes); // [2, 3]
    
    // 3. 大缓冲区读取（大于剩余数据）
    let mut big_buf = [0; 10];
    let n = cursor.read(&mut big_buf)?;
    println!("读取到大缓冲区: {:?}", &big_buf[..n]); // [4, 5]
    println!("实际读取字节数: {}", n);  // 2
    
    Ok(())
}
```


一些重要的特点：

1. **读取大小受限于缓冲区**：
```rust
let mut cursor = Cursor::new(vec![1, 2, 3, 4, 5]);
let mut small_buf = [0; 1];

cursor.read(&mut small_buf)?;
println!("只读取了一个字节: {:?}", small_buf); // [1]
```


2. **读取位置会前进**：
```rust
let mut cursor = Cursor::new(vec![1, 2, 3]);
let mut buf = [0; 1];

cursor.read(&mut buf)?;
println!("位置1: {}", cursor.position()); // 1

cursor.read(&mut buf)?;
println!("位置2: {}", cursor.position()); // 2
```


3. **可以重置位置重新读取**：
```rust
let mut cursor = Cursor::new(vec![1, 2, 3]);
let mut buf = [0; 2];

cursor.read(&mut buf)?;
println!("第一次读取: {:?}", buf); // [1, 2]

cursor.seek(SeekFrom::Start(0))?;
cursor.read(&mut buf)?;
println!("重新读取: {:?}", buf);  // [1, 2]
```


4. **读取到末尾的情况**：
```rust
let mut cursor = Cursor::new(vec![1, 2]);
let mut buf = [0; 3];

let n = cursor.read(&mut buf)?;
println!("读取的字节数: {}", n);     // 2
println!("缓冲区内容: {:?}", buf);   // [1, 2, 0]
```

这个机制使得我们可以：
- 灵活控制每次读取的数据量
- 在需要时重复读取数据
- 精确知道读取了多少数据
- 处理大小不匹配的情况


## b






`b` 前缀表示这是一个字节字面量（byte literal），它创建一个 `u8` 类型的值。在 Rust 中，`b` 前缀用于表示 ASCII 字符的字节值。

例如：
```rust
// 字节字面量
let byte1: u8 = b'+';  // ASCII 值为 43
let byte2: u8 = b'-';  // ASCII 值为 45
let byte3: u8 = b'*';  // ASCII 值为 42

// 比较
println!("{}", b'+' == 43);  // true
println!("{}", b'-' == 45);  // true

// 字节字符串字面量
let bytes = b"Hello";  // &[u8; 5]
```


在 Redis 协议解析中的应用：
```rust
impl Frame {
    pub fn parse(buf: &mut Cursor<&[u8]>) -> Result<Frame> {
        // buf.get_u8() 返回一个 u8，所以需要与字节字面量比较
        match buf.get_u8() {
            b'+' => {  // 43u8，表示简单字符串
                let string = get_line(buf)?;
                Ok(Frame::Simple(string))
            }
            b'-' => {  // 45u8，表示错误
                let string = get_line(buf)?;
                Ok(Frame::Error(string))
            }
            b':' => {  // 58u8，表示整数
                let num = get_decimal(buf)?;
                Ok(Frame::Integer(num))
            }
            b'$' => {  // 36u8，表示批量字符串
                // ...
            }
            b'*' => {  // 42u8，表示数组
                // ...
            }
            actual => {
                Err(format!("invalid byte: {}", actual).into())
            }
        }
    }
}
```


字节字面量与字符字面量的区别：
```rust
let byte = b'A';    // u8 类型，值为 65
let char = 'A';     // char 类型，4字节 Unicode 字符

// 字节字符串与普通字符串
let bytes = b"Hello";  // &[u8; 5]，ASCII 字节数组
let string = "Hello";  // &str，UTF-8 字符串

// 只能用于 ASCII 字符
// let invalid = b'世';  // 错误：非 ASCII 字符不能用作字节字面量
```


常见用途：
1. **协议解析**：
```rust
fn parse_protocol(byte: u8) {
    match byte {
        b'\r' => println!("回车符"),
        b'\n' => println!("换行符"),
        b' ' => println!("空格"),
        _ => println!("其他字符"),
    }
}
```


2. **二进制数据处理**：
```rust
let header = [b'P', b'N', b'G'];  // 文件魔数
let data = vec![0u8; 100];

if data.starts_with(&header) {
    println!("这是 PNG 文件");
}
```


3. **ASCII 字符集操作**：
```rust
fn is_digit(byte: u8) -> bool {
    byte >= b'0' && byte <= b'9'
}

fn is_uppercase(byte: u8) -> bool {
    byte >= b'A' && byte <= b'Z'
}
```


4. **网络协议**：
```rust
const HTTP_GET: &[u8] = b"GET";
const HTTP_POST: &[u8] = b"POST";

fn parse_http_method(data: &[u8]) {
    match data {
        HTTP_GET => println!("GET 请求"),
        HTTP_POST => println!("POST 请求"),
        _ => println!("其他请求"),
    }
}
```


总结：
- `b` 前缀创建字节字面量（`u8` 类型）
- 只能用于 ASCII 字符
- 常用于二进制协议解析
- 与字符字面量（`char`）不同
- 在处理二进制数据时很有用



## tokio

### tokio 中的 async 与 await

The async fn definition looks like a regular synchronous function, but operates asynchronously. Rust transforms the async fn at compile time into a routine that operates asynchronously. Any calls to .await within the async fn yield control back to the thread. The thread may do other work while the operation processes in the background.

async 函数调用 .await 时，会将控制权让给 tokio 调度器，调度器会根据当前的运行状态，决定是否将控制权让给其他任务。

#### Rust lazy async/await


```javascript
// JavaScript 中的 async 函数
async function fetchData() {
    console.log("开始");
    // Promise 会立即开始执行
    const response = await fetch('https://api.example.com/data');
    console.log("结束");
    return response.json();
}

// 调用函数
fetchData(); // 立即执行
```


```rs
async fn fetch_data() {
    println!("开始");
    // 这里只是创建了 Future，还没有开始执行
    let response = reqwest::get("https://api.example.com/data");
    println!("结束");
    response.await.json().await
}

#[tokio::main]
async fn main() {
    // 创建 Future，但还没有开始执行
    let future = fetch_data();
    
    // 只有在 .await 时才真正开始执行
    future.await;
}
```

- 在 Rust 中，async 函数在定义时不会立即执行，只有在调用 .await 时才会真正开始执行。


#### .await 与 poll

让我来解释 `.await` 和 `poll` 的关系：

1. **`.await` 机制**
- 当你使用 `.await` 时，Rust 编译器会将代码转换为状态机
- 这个状态机会通过 `Future` trait 的 `poll` 方法来驱动异步操作

2. **编译器展开**
当你写这样的代码：
```rust
async fn main() {
    let future = Delay { when };
    let out = future.await;
}
```

编译器实际上会将其转换为类似这样的状态机：
```rust
// 这是简化的伪代码，展示原理
enum MainFuture {
    State0(Delay),
    State1,
    Completed,
}

impl Future for MainFuture {
    type Output = ();
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        loop {
            match self {
                State0(delay) => {
                    // 调用内部 Future 的 poll
                    match delay.poll(cx) {
                        Poll::Ready(output) => {
                            // 转移到下一个状态...
                            self.set(State1);
                            continue;
                        }
                        Poll::Pending => return Poll::Pending,
                    }
                }
                // 其他状态处理...
            }
        }
    }
}
```

3. **关键点**
- `Future` trait 确实定义了 `poll` 方法作为必须实现的方法
- `.await` 是编译器层面的语法糖，它会自动生成调用 `poll` 的代码
- 这个机制允许异步运行时（如 tokio）来控制 Future 的执行

4. **执行流程**
1. 当遇到 `.await` 时，当前的 Future 会被注册到异步运行时
2. 运行时会调用 Future 的 `poll` 方法
3. 如果返回 `Poll::Pending`，运行时会保存这个 Future 并等待被唤醒
4. 当 `waker` 被触发时，运行时会再次调用 `poll`
5. 直到获得 `Poll::Ready` 结果

所以，`.await` 和 `poll` 的关系是：`.await` 是高层语法，它通过编译器生成的代码来调用底层的 `poll` 方法。这不是 `Future` trait 的默认行为，而是 Rust 异步编程的核心设计。


#### waker 的粗暴理解


当在 poll 中调用 cs.waker().wake_ny_ref() 时，运行时就会在 loop 中一直使用 match 来匹配 self 的状态了

```rust
// 这是简化的运行时实现伪代码
struct Runtime {
    // 待处理的 Future 队列
    pending_futures: Vec<(Future, Waker)>,
    // 已经被唤醒需要重新检查的 Future 队列
    woken_futures: Vec<Future>,
}

impl Runtime {
    fn run(&mut self) {
        loop {
            // 1. 检查所有被唤醒的 futures
            for future in self.woken_futures.drain(..) {
                match future.poll() {
                    Poll::Ready(result) => {
                        // Future 完成，处理结果
                    },
                    Poll::Pending => {
                        // 当 future 调用 wake_by_ref() 时
                        // future 会被重新加入 pending_futures
                        self.pending_futures.push((future, waker));
                    }
                }
            }

            // 2. 如果没有被唤醒的 futures，则等待
            if self.woken_futures.is_empty() {
                // 等待直到某个 waker 被触发
                self.wait_for_wakeup();
            }
        }
    }
}

// 当 waker.wake() 被调用时
impl Waker {
    fn wake(&self) {
        // 将对应的 future 从 pending_futures 移动到 woken_futures
        let future = /* 从 pending_futures 找到对应的 future */;
        runtime.woken_futures.push(future);
    }
}
```

这个模型中：

1. **Future 状态管理**
- 未完成的 Future 存在 `pending_futures` 中
- 被唤醒的 Future 会被移到 `woken_futures` 中
- 运行时主循环会不断检查 `woken_futures`

2. **wake_by_ref() 的作用**
```rust
impl Future for Delay {
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<&'static str> {
        if Instant::now() >= self.when {
            Poll::Ready("done")
        } else {
            cx.waker().wake_by_ref();  // 将自己注册到 pending_futures
            Poll::Pending              // 返回 Pending，等待下次被唤醒
        }
    }
}
```

3. **执行流程**
1. Future 返回 `Pending` 时，会被放入 `pending_futures`
2. 当条件满足时（如 I/O 完成、时间到达），调用 `wake()`
3. `wake()` 将 Future 移动到 `woken_futures`
4. 运行时主循环检测到 `woken_futures` 有内容，继续处理

所以你的理解基本正确，只是有两点补充：
- 实际上是两个队列在配合工作（pending 和 woken）
- 运行时不会一直轮询所有 Future，而是只处理被唤醒的 Future，这样更高效

这种机制保证了：
- 高效的资源使用（不会忙等）
- 及时的任务处理（被唤醒后立即处理）
- 良好的并发性能（可以处理大量 Future）



### spanw

- 任务是轻量级的
- 任务可以并发执行
- 任务可以在线程间移动(任务中的状态均需实现 Send)
- 任务必须是 'static
- 任务的生命周期独立于创建它的作用域

### std::sync::Mutex 与 tokio::sync::Mutex

- std::sync::Mutex：
    - 会阻塞当前线程直到获取锁
    - 适用于短时间持有锁的场景
    - 在低竞争情况下性能更好

- tokio::sync::Mutex：
    - 设计用于跨越 .await 点持有锁
    - 内部仍然使用同步互斥锁
    - 增加了额外的异步开销


### waker 需要满足的要求


waker 需要满足的核心要求是：当一个 Future 返回 `Poll::Pending` 时，必须确保在未来某个时刻当该 Future 可以取得进展时，会调用 waker 来通知执行器。

让我们通过代码来理解：

```rust
impl Future for MySelect {
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        // 将 cx (包含 waker) 传递给内部 futures
        if let Poll::Ready(val) = Pin::new(&mut self.rx1).poll(cx) {
            return Poll::Ready(());
        }

        if let Poll::Ready(val) = Pin::new(&mut self.rx2).poll(cx) {
            return Poll::Ready(());
        }

        // 只有当内部 futures 都返回 Pending 时才返回 Pending
        // 此时内部 futures 已经保存了 waker，并会在就绪时调用
        Poll::Pending
    }
}
```

waker 要求具体包括：

1. **保存 waker**：
   - Future 在返回 `Pending` 时必须保存当前的 waker
   - 确保可以在将来使用它来通知执行器

2. **及时通知**：
   - 当 Future 可以继续执行时，必须调用 waker
   - 不能遗漏通知，否则任务可能永远挂起

3. **正确传播**：
   - 复合 Future (如 MySelect) 必须正确传递 waker 给内部 futures
   - 确保通知可以正确传播到执行器

如果不满足这些要求，可能导致任务永远挂起，因为执行器不知道何时需要重新轮询该 Future。

### select
q: 会不会出现以下情况：
rx2状态改为 ready 时 poll 方法并未执行，rx1 状态改为 ready 时，poll 方法执行

a: 这种情况是可能发生的。让我们看看相关代码：


```11:27:redis/examples/select.rs
impl Future for MySelect {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if let Poll::Ready(val) = Pin::new(&mut self.rx1).poll(cx) {
            println!("rx1 completed first with {:?}", val);
            return Poll::Ready(());
        }

        if let Poll::Ready(val) = Pin::new(&mut self.rx2).poll(cx) {
            println!("rx2 completed first with {:?}", val);
            return Poll::Ready(());
        }

        Poll::Pending
    }
}
```


在这个实现中，`poll` 方法是按顺序检查 rx1 和 rx2 的。如果在某个时刻：

1. rx2 变为 Ready
2. 但在运行时调用 `poll` 之前，rx1 也变为 Ready
3. 当 `poll` 执行时，会先检查 rx1，发现它 Ready
4. rx1 的分支会被执行，而 rx2 的 Ready 状态会被忽略

这就是为什么实际的 `select!` 宏的实现会随机选择轮询顺序，以避免总是偏向第一个分支。让我们看看文档中提到的这一点：


```517:521:note/q.md
3. **关键点**
- `Future` trait 确实定义了 `poll` 方法作为必须实现的方法
- `.await` 是编译器层面的语法糖，它会自动生成调用 `poll` 的代码
- 这个机制允许异步运行时（如 tokio）来控制 Future 的执行

```


为了解决这个问题，tokio 的 `select!` 宏实际实现会：

1. 随机化轮询顺序
2. 确保公平性
3. 避免饥饿问题

这样就能确保所有分支都有公平的机会被执行，而不是总是偏向于代码中排在前面的分支。

#### 异步表达式阶段与处理器阶段
不矛盾，这里说的是两个不同的阶段：

1. **异步表达式阶段**（并发执行）：

When the select macro is evaluated, all the <async expression>s are aggregated and executed concurrently.
```rust
tokio::select! {
    // 这些异步表达式会并发执行
    res1 = async { ... } => handler1(),
    res2 = async { ... } => handler2(),
}
```

2. **处理器阶段**（互斥执行）：
When it comes to each branch's <handler>, select! guarantees that only a single <handler> runs
```rust
tokio::select! {
    _ = async_op1() => {
        // 只有一个处理器会执行
        // 如果这个处理器执行，另一个就不会执行
        mut_data.modify();  // 可以安全地可变借用
    }
    _ = async_op2() => {
        // 这个处理器和上面的处理器不会同时执行
        mut_data.modify();  // 也可以安全地可变借用
    }
}
```

执行流程：
1. 所有 `async { ... }` 表达式并发执行
2. 当其中一个完成时，其对应的处理器被执行
3. 其他异步表达式被取消，它们的处理器不会执行

#### 随机选择 与 消息保留
- 随机选择

以上中也提到了当在一次 select 中，如果多个异步表达式同时完成，那么会随机选择一个执行；以下再说以下

```rust
async fn example() {
    let (tx1, mut rx1) = mpsc::channel(1);
    let (tx2, mut rx2) = mpsc::channel(1);
    
    // 假设两个通道都有消息
    tx1.send(1).await.unwrap();
    tx2.send(2).await.unwrap();
    
    // select! 会随机选择一个通道
    tokio::select! {
        msg = rx1.recv() => println!("rx1: {:?}", msg),
        msg = rx2.recv() => println!("rx2: {:?}", msg),
    }
    // 另一个消息仍然在通道中
}
```
select! 宏随机选择首先检查就绪状态的分支。当多个通道有待处理的值时，会随机选择一个通道来接收。这是为了处理接收循环处理消息的速度比它们被推入通道的速度慢的情况，意味着通道开始填满。如果 select! 不随机选择首先检查的分支，在循环的每次迭代中，都会先检查 rx1。如果 rx1 总是包含新消息，其余通道就永远不会被检查。
如果当 select! 被求值时，多个通道有待处理的消息，只有一个通道的值会被弹出。所有其他通道保持不变，它们的消息保留在这些通道中，直到下一次循环迭代。不会丢失任何消息。

- 消息保留
```rust
async fn action() -> i32 {
    // Some asynchronous logic
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    println!("action");
    return 1;
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(128);    
    
    tx.send(1).await.unwrap();
    tx.send(2).await.unwrap();

    // 在 loop 之外创建 future ，在 loop 内部使用，确保是同一个 future
    let operation = action();
    // 异步任务发生移动时，内部自引用会失效，所以要 Pin, Pin 是 .await 的必要条件
    tokio::pin!(operation);
    
    loop {
        tokio::select! {
            // 不用显示调用 await ，select 内部会尝试推进异步任务
            /*
            * 需要 &mut 原因如下：
            * 1. Future 在执行过程中需要修改其内部状态
            * 2. 不可变引用（&）不允许修改状态
            * 3. poll 方法需要 Pin<&mut Self> 来推进 Future 的执行
            */
            _ = &mut operation => break,
            Some(v) = rx.recv() => {
                if v % 2 == 0 {
                    break;
                }
            }
        }
    }
}
```

#### future 的消耗性

通常的 future 是消耗性的，即一旦被使用，就不能再被使用。










让我详细解释为什么会 panic：

1. **普通的 await 行为**：
```rust
async fn example() {
    let future = async { 42 };
    let result = future.await;    // future 被消耗
    // let result2 = future.await;  // ❌ 错误：future 已被消耗
}
```


2. **通过引用 await 的行为**：
```rust
async fn example() {
    let future = async { 42 };
    let mut pinned = Box::pin(future);
    
    let result = (&mut pinned).await;  // future 没有被消耗
    let result2 = (&mut pinned).await; // ❌ panic: 尝试重用已完成的 future
}
```


3. **在 select! 中的情况**：
```rust
let operation = action(None);
tokio::pin!(operation);

loop {
    tokio::select! {
        res = &mut operation => {  // 通过引用使用 operation
            // operation 完成了，但还存在
        }
        // 下次循环会再次尝试 poll 已完成的 operation
        // 这会导致 panic
    }
}
```


原因是：
- Future 设计为一次性使用
- 通过引用 await 不会消耗 Future
- 重复 poll 已完成的 Future 是不安全的
- 这就像尝试从已关闭的文件读取数据

所以需要使用 `done` 标志：
```rust
let mut done = false;
loop {
    tokio::select! {
        res = &mut operation, if !done => {  // 只有未完成时才尝试 poll
            done = true;
        }
    }
}
```

q: 为什么下一次循环不会异常？

```rust
loop {
        tokio::select! {
            res = &mut operation, if !done => {
                done = true;

                if let Some(v) = res {
                    println!("GOT = {}", v);
                    return;
                }
            }
            Some(v) = rx.recv() => {
                if v % 2 == 0 {
                    // `.set` is a method on `Pin`.
                    operation.set(action(Some(v)));
                    done = false;
                }
            }
        }
    }
```




让我解释为什么下一次循环不会异常：

1. **关键在于 `operation.set()`**：
```rust
loop {
    tokio::select! {
        res = &mut operation, if !done => {
            done = true;  // 标记完成
        }
        Some(v) = rx.recv() => {
            if v % 2 == 0 {
                // 重要！创建新的 Future 并替换旧的
                operation.set(action(Some(v)));  
                done = false;  // 重置完成标志
            }
        }
    }
}
```


2. **Pin::set 的作用**：
```rust
impl<P> Pin<P> {
    pub fn set(&mut self, value: T) {
        // 替换被固定的值，但保持 Pin 不变
        // 相当于创建了一个全新的 Future
    }
}
```


执行流程：
1. 第一次循环：
```rust
// operation 完成
res = &mut operation, if !done => {
    done = true;  // 标记完成
}
```


2. 收到新值时：
```rust
Some(v) = rx.recv() => {
    if v % 2 == 0 {
        operation.set(action(Some(v)));  // 创建新的 Future
        done = false;  // 重置标志
    }
}
```


3. 下一次循环：
```rust
// operation 是全新的 Future，可以安全使用
res = &mut operation, if !done => {
    // ...
}
```


所以不会异常，因为：
- 不是重用已完成的 Future
- 而是用新的 Future 替换了旧的
- `done` 标志也被重置



#### select 与 spawn 的并发对比


**每个任务的并发性**

`tokio::spawn` 和 `select!` 都能运行并发的异步操作。但是，它们用于运行并发操作的策略不同：

1. **tokio::spawn 的特点**：
```rust
// 创建新任务
tokio::spawn(async {
    // 这是一个独立的任务
    // 可能在不同的 OS 线程上运行
    // 不能借用外部数据
});
```



2. **select! 的特点**：
```rust
tokio::select! {
    // 所有分支在同一个任务中运行
    _ = async_op1() => {},
    _ = async_op2() => {},
    // 永远不会同时运行
}
```



3. **对比示例**：
```rust
async fn example() {
    let data = vec![1, 2, 3];
    
    // spawn 方式 - 需要所有权
    tokio::spawn(async move {
        process_data(data).await  // data 被移动到新任务
    });
    
    // select 方式 - 可以借用
    tokio::select! {
        _ = process_data(&data) => {},  // 可以借用 data
        _ = backup_data(&data) => {},   // 可以同时借用
    }
}
```



4. **任务调度的区别**：
```rust
// spawn 创建多个独立任务
tokio::spawn(async { task1().await });
tokio::spawn(async { task2().await });
// 可能同时在不同线程运行

// select 在单个任务中多路复用
tokio::select! {
    _ = task1() => {},
    _ = task2() => {},
}
// 永远在同一个任务中交替执行
```



关键区别：

1. **执行模型**：
   - spawn: 创建新任务，可能并行执行
   - select: 在同一任务中交替执行

2. **数据访问**：
   - spawn: 需要所有权，不能借用
   - select: 可以借用数据

3. **资源使用**：
   - spawn: 每个任务都有独立的资源
   - select: 共享同一个任务的资源

4. **使用场景**：
   - spawn: 适合独立的长期运行任务
   - select: 适合需要协调的多个操作

示例应用：

1. **使用 spawn**：
```rust
// 独立的后台任务
tokio::spawn(async {
    loop {
        cleanup_database().await;
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }
});
```



2. **使用 select**：
```rust
// 需要协调的操作
tokio::select! {
    res = process_request(&data) => {
        handle_response(res);
    }
    _ = timeout(Duration::from_secs(5)) => {
        handle_timeout();
    }
}
```



选择建议：
1. 使用 spawn 当：
   - 需要真正的并行执行
   - 任务相互独立
   - 不需要共享数据

2. 使用 select 当：
   - 需要协调多个操作
   - 需要共享数据
   - 需要取消或超时处理


 
### 局部异步代码

#### block_on 创建运行时

```rust
let rt = Runtime::new().unwrap();
rt.block_on(async {
    // 这里运行在运行时A的上下文中
});
```
#### block_on 的使用问题

block_on 会阻塞当前线程，直到异步任务完成。

在以下情况下使用 block_on 会导致问题：
在任何已有 Tokio 运行时上下文中使用，包括：
   - 异步函数上下文
   - Tokio 任务中
   - 其他由 Tokio 运行时管理的代码

本质原因：
- Tokio 不允许在一个运行时上下文中创建并使用新的运行时

#### Runtime spawn

在运行时上创建新的后台任务
```rust
use tokio::runtime::Builder;
let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

runtime.spawn(async {
    // 这里运行在运行时A的上下文中
});
```
#### 在独立的线程中运行 Runtime, 通过消息传递进行通信

适用于需要保持主线程响应性的场景

### 优雅关闭

There are usually three parts to implementing graceful shutdown:

1. Figuring out when to shut down.
2. Telling every part of the program to shut down.
3. Waiting for other parts of the program to shut down.
