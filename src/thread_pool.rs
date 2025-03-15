//Problem statement
//we need to execute multiple tasks concurrently
//creating a new thread for each task is expensive
//We want to limit the maximum number of threads running simultaneously
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};
type Job = Box<dyn FnOnce() + Send + 'static>;
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

pub struct Worker {
    _id: usize,
    _thread: Option<thread::JoinHandle<()>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        let (sender, reciever) = mpsc::channel();
        let reciever = Arc::new(Mutex::new(reciever));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&reciever)));
        }
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }
    //excute the function

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Worker {
    pub fn new(id: usize, recevier: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = recevier.lock().unwrap().recv();
            match message {
                Ok(job) => {
                    job();
                }
                Err(_) => {
                    break;
                }
            }
        });
        Worker {
            _id: id,
            _thread: Some(thread),
        }
    }
}

impl Drop for ThreadPool{
    fn drop(&mut self) {
        drop(self.sender.take());
    }
}
