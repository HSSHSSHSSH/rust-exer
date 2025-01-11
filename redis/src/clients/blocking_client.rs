use tokio::net::ToSocketAddrs;
use tokio::runtime::Runtime;

pub use crate::clients::client::Message;

/// Established connection with a Redis server.
pub struct BlockingClient {
    /// The asynchronous `Client`.
    inner: crate::clients::Client,

    /// A `current_thread` runtime for executing operations on the
    /// asynchronous client in a blocking manner.
    rt: Runtime,
}

impl BlockingClient {
    pub fn connect<T: ToSocketAddrs>(addr: T) -> crate::Result<BlockingClient> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        // Call the asynchronous connect method using the runtime.
        let inner = rt.block_on(crate::clients::Client::connect(addr))?;

        Ok(BlockingClient { inner, rt })
    }
}


async fn deadlock_example() {
    // 这里运行在运行时A的上下文中
    let rt = Runtime::new().unwrap();
    
    // block_on 会阻塞运行时A正在使用的线程
    rt.block_on(std::thread::spawn(async {
        println!("...");
    }));
}