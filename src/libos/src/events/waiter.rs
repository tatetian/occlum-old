use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Weak;
use std::time::Duration;

use super::host_event_fd::HostEventFd;
use crate::prelude::*;

/// A waiter enables a thread to sleep.
pub struct Waiter {
    inner: Arc<Inner>,
}

// Waiter is bound to the thread that creates it. So it cannot be sent to
// or accessed by another thread.
impl !Send for Waiter {}
impl !Sync for Waiter {}

impl Waiter {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner::new()),
        }
    }

    pub fn is_woken(&self) -> bool {
        self.inner.is_woken()
    }

    pub fn reset(&self) {
        self.inner.reset();
    }

    pub fn wait(&self, timeout: Option<&Duration>) -> Result<()> {
        self.inner.wait(timeout)
    }

    pub fn wait_mut(&self, timeout: Option<&mut Duration>) -> Result<()> {
        self.inner.wait_mut(timeout)
    }

    pub fn waker(&self) -> Waker {
        Waker {
            inner: Arc::downgrade(&self.inner),
        }
    }
}

/// A waker can wake up the thread that its waiter has put to sleep.
pub struct Waker {
    inner: Weak<Inner>,
}

impl Waker {
    pub fn wake(&self) {
        if let Some(inner) = self.inner.upgrade() {
            inner.wake()
        }
    }

    pub fn batch_wake<'a, I: Iterator<Item = &'a Waker>>(iter: I) {
        Inner::batch_wake(iter);
    }
}

struct Inner {
    is_woken: AtomicBool,
    host_eventfd: Arc<HostEventFd>,
}

impl Inner {
    pub fn new() -> Self {
        let is_woken = AtomicBool::new(false);
        let host_eventfd = current!().host_eventfd().clone();
        Self {
            is_woken,
            host_eventfd,
        }
    }

    pub fn is_woken(&self) -> bool {
        self.is_woken.load(Ordering::SeqCst)
    }

    pub fn reset(&self) {
        self.is_woken.store(false, Ordering::SeqCst);
    }

    pub fn wait(&self, timeout: Option<&Duration>) -> Result<()> {
        while !self.is_woken() {
            self.host_eventfd.poll(timeout)?;
        }
        Ok(())
    }

    pub fn wait_mut(&self, timeout: Option<&mut Duration>) -> Result<()> {
        let mut remain = timeout.as_ref().map(|d| **d);

        // Need to change timeout from `Option<&mut Duration>` to `&mut Option<Duration>`
        // so that the Rust compiler is happy about using the variable in a loop.
        let ret = self.do_wait_mut(&mut remain);

        if let Some(timeout) = timeout {
            *timeout = remain.unwrap();
        }
        ret
    }

    fn do_wait_mut(&self, remain: &mut Option<Duration>) -> Result<()> {
        while !self.is_woken() {
            self.host_eventfd.poll_mut(remain.as_mut())?;
        }
        Ok(())
    }

    pub fn wake(&self) {
        if self
            .is_woken
            .compare_and_swap(false, true, Ordering::SeqCst)
            == false
        {
            self.host_eventfd.write_u64(1);
        }
    }

    pub fn batch_wake<'a, I: Iterator<Item = &'a Waker>>(iter: I) {
        let host_eventfds = iter
            .filter_map(|waker| waker.inner.upgrade())
            .filter(|inner| {
                inner
                    .is_woken
                    .compare_and_swap(false, true, Ordering::SeqCst)
                    == false
            })
            .map(|inner| inner.host_eventfd.host_fd())
            .collect::<Vec<FileDesc>>();
        unsafe {
            HostEventFd::write_u64_raw_and_batch(&host_eventfds, 1);
        }
    }
}
