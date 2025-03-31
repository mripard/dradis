use std::{
    collections::VecDeque,
    thread::{self, JoinHandle},
};

use tracing::warn;

pub struct ThreadPool<T> {
    queue: VecDeque<JoinHandle<T>>,
    limit: Option<usize>,
}

impl<T> ThreadPool<T>
where
    T: Send + 'static,
{
    pub fn new(limit: Option<usize>) -> Self {
        Self {
            queue: VecDeque::new(),
            limit,
        }
    }

    pub fn spawn_and_queue<F>(&mut self, f: F)
    where
        F: FnOnce() -> T + Send + 'static,
    {
        if let Some(limit) = self.limit {
            if self.queue.len() >= limit {
                return;
            }
        }

        self.queue.push_back(thread::spawn(f));
    }
}

impl<T> Drop for ThreadPool<T> {
    fn drop(&mut self) {
        while let Some(th) = self.queue.pop_front() {
            if let Err(_) = th.join() {
                warn!("Error joining frame dumps threads.");
            }
        }
    }
}
