use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use super::host_event_fd::HostEventFd;
use crate::prelude::*;

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
            inner: self.inner.clone(),
        }
    }
}

pub struct Waker {
    inner: Arc<Inner>,
}

impl Waker {
    pub fn wake(&self) {
        self.inner.wake()
    }

    pub fn batch_wake<'a, I: Iterator<Item = &'a Waker>>(iter: I) {
        // TODO
    }
}

struct Inner {
    is_woken: AtomicBool,
    host_eventfd: Arc<HostEventFd>,
}

impl Inner {
    pub fn new() -> Self {
        let is_woken = AtomicBool::new(false);
        // TODO: remove host_evenfd from Thread
        let host_eventfd = HOST_EVENTFD.with(|fd| fd.clone());
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

        let mut poll_loop = || -> Result<()> {
            while !self.is_woken() {
                self.host_eventfd.poll_mut(remain.as_mut())?;
            }
            Ok(())
        };
        let ret = poll_loop();

        if let Some(timeout) = timeout {
            *timeout = remain.unwrap();
        }
        ret
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
}

// Each thread is associated a HostEventFd.
thread_local!(static HOST_EVENTFD: Arc<HostEventFd> = Arc::new(HostEventFd::new().unwrap()));
