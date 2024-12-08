use std::{
  thread,
  sync::{mpsc, Arc, Mutex},
};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
  threads: Vec<Worker>,
  sender: mpsc::Sender<Job>,
}

impl ThreadPool {
  pub fn new(size: usize) -> ThreadPool {
    assert!(size > 0);
    let (sender, receiver) = mpsc::channel();
    let receiver = Arc::new(Mutex::new(receiver));
    let mut threads = Vec::with_capacity(size);
    for id in 0..size {
      threads.push(Worker::new(id, Arc::clone(&receiver)));
    }
    ThreadPool { threads, sender }
  }

  pub fn execute<F>(&self, _f: F)
  where F: FnOnce() + Send + 'static {
    let job = Box::new(_f);
    self.sender.send(job).unwrap();
  }
}


struct Worker {
  id: usize,
  thread: thread::JoinHandle<()>,
}

impl Worker {
  pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
    let thread = thread::spawn(move || loop {
      let job = receiver.lock().unwrap().recv().unwrap();
      println!("Worker {} got a job; executing.", id);
      job();
    });
    Worker { id, thread }
  }
} 
