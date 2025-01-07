use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};
use futures::task;
use std::thread;
use std::sync::{Arc, mpsc, Mutex};
use futures::task::ArcWake;

fn main() {
    let mut mini_tokio = MiniTokio::new();

    mini_tokio.spawn(async {
        let when = Instant::now() + Duration::from_millis(10);
        let future = Delay { when };

        let out = future.await;
        assert_eq!(out, "done");
    });

    mini_tokio.run();
}

struct MiniTokio {
    /**
     * VecDeque 是一个双端队列，允许在队列的两端进行高效的插入和删除操作。
     * 它实现了 Deque 接口，提供了高效的插入、删除和随机访问操作。
     * 
     * 在 MiniTokio 中，tasks 是一个 VecDeque，用于存储待执行的任务。
     * 每个任务都是一个 Future，当任务完成时，会从队列中移除。
     * 常用 api 有： push_back, push_front, pop_front, pop_back, is_empty, len, front, back 等。
     */
    // tasks: VecDeque<Task>,
    scheduled: mpsc::Receiver<Arc<Task>>,
    sender: mpsc::Sender<Arc<Task>>,
}

struct TaskFuture {
    future: Pin<Box<dyn Future<Output = ()> + Send>>,
    poll: Poll<()>,
}

impl TaskFuture {
    /**
     * 创建一个新的 TaskFuture，将 future 包装在 Box 中，并初始化 poll 为 Pending。
     */
    fn new(future: impl Future<Output = ()> + Send + 'static) -> TaskFuture {
        TaskFuture {
            future: Box::pin(future),
            poll: Poll::Pending,
        }
    }

    fn poll(&mut self, cx: &mut Context<'_>) {
        // Spurious wake-ups are allowed, even after a future has                                  
        // returned `Ready`. However, polling a future which has                                   
        // already returned `Ready` is *not* allowed. For this                                     
        // reason we need to check that the future is still pending                                
        // before we call it. Failure to do so can lead to a panic.
        if self.poll.is_pending() {
            self.poll = self.future.as_mut().poll(cx);
        }
    }
}

struct Task {
    task_future: Mutex<TaskFuture>,
    executor: mpsc::Sender<Arc<Task>>,
}

impl Task {

    fn schedule(self: &Arc<Self>) {
        self.executor.send(self.clone()).unwrap();
    }
    
    fn poll(self: Arc<Self>) {
        // Create a waker from the `Task` instance. This
        // uses the `ArcWake` impl from above.
        let waker = task::waker(self.clone());
        let mut cx = Context::from_waker(&waker);

        // No other thread ever tries to lock the task_future
        let mut task_future = self.task_future.try_lock().unwrap();

        // Poll the inner future
        task_future.poll(&mut cx);
    }

    fn spawn(future: impl Future<Output = ()> + Send + 'static, executor: &mpsc::Sender<Arc<Task>>) {
        let task = Arc::new(Task {
            task_future: Mutex::new(TaskFuture::new(future)),
            executor: executor.clone(),
        });
        executor.send(task).unwrap();
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.schedule();
    }
}


impl MiniTokio {
    fn new() -> MiniTokio {
        let (sender, scheduled) = mpsc::channel();
        MiniTokio {
            scheduled,
            sender,
        }
    }

    /// Spawn a future onto the mini-tokio instance.
    fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Task::spawn(future, &self.sender);
    }

    fn run(&mut self) {
        while let Ok(task) = self.scheduled.recv() {
            println!("Polling task");
            task.poll();
        }
    }

    
}


struct Delay {
    when: Instant,
    waker: Option<Arc<Mutex<Waker>>>,
}

impl Future for Delay {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>)
        -> Poll<()>
    {
        

        if Instant::now() >= self.when {
            println!("Hello world");
            Poll::Ready(())
        } else {
            // Get a handle to the waker for the current task
            let waker = cx.waker().clone();
            let when = self.when;

            // Spawn a timer thread.
            thread::spawn(move || {
                let now = Instant::now();

                if now < when {
                    thread::sleep(when - now);
                }

                waker.wake();
            });

            Poll::Pending
        }
    }
}