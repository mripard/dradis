//! Thread Spawner with optional limit Library

extern crate alloc;

use alloc::collections::VecDeque;
use std::thread::{self, JoinHandle};

use tracing::warn;

/// A Thread dispatcher, with an optional limit on the number of threads that can be spawned.
#[derive(Debug)]
pub struct ThreadPool<T> {
    queue: VecDeque<JoinHandle<T>>,
    limit: Option<usize>,
}

impl<T> ThreadPool<T>
where
    T: Send + 'static,
{
    /// Creates a new [`ThreadPool`] instance.
    #[must_use]
    pub fn new(limit: Option<usize>) -> Self {
        Self {
            queue: VecDeque::new(),
            limit,
        }
    }

    /// Spawns a thread that will run the given closure, if we are not over the limit.
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
            if th.join().is_err() {
                warn!("Error joining frame dumps threads.");
            }
        }
    }
}
