use std::{
    sync::{Arc, Mutex},
    thread,
};

use crossbeam_channel::{Receiver, Sender};

pub struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

type Task = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Task>>>) -> Self {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv();
                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job; executing.");
                        job();
                    }
                    Err(_) => {
                        println!("Worker {id} disconnected; shutting down.");
                        break;
                    }
                }
            }
        });
        Self { id, thread }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Task>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let mut workers = Vec::with_capacity(size);
        let (tx, rx) = crossbeam_channel::bounded::<Task>(100);
        let receiver = Arc::new(Mutex::new(rx));
        for id in 0..size {
            let receiver_clone = Arc::clone(&receiver);
            workers.push(Worker::new(id, receiver_clone));
        }
        ThreadPool {
            workers,
            sender: Some(tx),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in self.workers.drain(..) {
            worker.thread.join().unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::threadpool::ThreadPool;

    #[test]
    fn test_threadpool() {
        let sum = |x: usize, y: usize| x + y;
        let thread = ThreadPool::new(4);
        thread.execute(move || {
            let result = sum(4, 5);
            println!("Sum is {}", result);
        });
    }
}
