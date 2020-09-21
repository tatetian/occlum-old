use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use super::{Waiter, Waker};
use crate::prelude::*;

pub struct WaiterQueue {
    count: AtomicUsize,
    wakers: SgxMutex<VecDeque<Waker>>,
}

impl WaiterQueue {
    pub fn new() -> Self {
        Self {
            count: AtomicUsize::new(0),
            wakers: SgxMutex::new(VecDeque::new()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count.load(Ordering::SeqCst) == 0
    }

    pub fn enqueue(&self, waiter: &Waiter) {
        let mut wakers = self.wakers.lock().unwrap();
        self.count.fetch_add(1, Ordering::SeqCst);
        wakers.push_back(waiter.waker());
    }

    pub fn dequeue_and_wake_one(&self) -> usize {
        self.dequeue_and_wake_nr(1)
    }

    pub fn dequeue_and_wake_all(&self) -> usize {
        self.dequeue_and_wake_nr(usize::MAX)
    }

    pub fn dequeue_and_wake_nr(&self, max_count: usize) -> usize {
        // The quick path for a common case
        if self.is_empty() {
            return 0;
        }

        // Dequeue wakers
        let to_wake = {
            let mut wakers = self.wakers.lock().unwrap();
            let to_wake: Vec<Waker> = wakers.drain(..max_count).collect();
            self.count.fetch_sub(to_wake.len(), Ordering::SeqCst);
            to_wake
        };

        // Wake in batch
        Waker::batch_wake(to_wake.iter());
        to_wake.len()
    }
}
