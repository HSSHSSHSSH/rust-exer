use tokio::sync::{mpsc, oneshot};
use bytes::Bytes;
use mini_redis::client;

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp: Responder<()>,
    }
}

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();

    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get { key: "foo".to_string(), resp: resp_tx };
        tx.send(cmd).await.unwrap();

        let result = resp_rx.await.unwrap().unwrap();
        println!("GOT11: {:?}", result);
    });

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Set { key: "foo".to_string(), val: Bytes::from("bar"), resp: resp_tx };
        tx2.send(cmd).await.unwrap();

        let result = resp_rx.await.unwrap().unwrap();
        println!("SET22: {:?}", result);
    });

    let manager = tokio::spawn(async move {
        // Establish a connection to the server
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();
        
        // Start receiving messages
        while let Some(cmd) = rx.recv().await {
            use Command::*;
    
            match cmd {
                Get { key, resp } => {
                    let result = client.get(&key).await;
                    resp.send(result).unwrap();
                }
                Set { key, val, resp } => {
                    let result = client.set(&key, val).await;
                    resp.send(result).unwrap();
                }
            }
        }
    });
    

    t2.await.unwrap();
    t1.await.unwrap();
    manager.await.unwrap();
}