use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().unwrap();
    // 死锁：在 tokio 任务中使用 block_on
    rt.block_on(async {
        println!("这里会死锁");
    });
}
