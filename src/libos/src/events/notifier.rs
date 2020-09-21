use std::any::Any;
use std::sync::Weak;

use super::{Event, EventFilter, Observer};
use crate::prelude::*;

// TODO: how to prevent deadlock or infinite loop during event propagation?

pub struct Notifier<E: Event, F: EventFilter<E>> {
    subscribers: SgxMutex<VecDeque<Subscriber<E, F>>>,
}

struct Subscriber<E: Event, F: EventFilter<E>> {
    observer: Weak<dyn Observer<E>>,
    filter: Option<F>,
    metadata: Option<Box<dyn Any>>,
}

impl<E: Event, F: EventFilter<E>> Notifier<E, F> {
    pub fn new() -> Self {
        let subscribers = SgxMutex::new(VecDeque::new());
        Self { subscribers }
    }

    pub fn add(
        &self,
        observer: Weak<dyn Observer<E>>,
        filter: Option<F>,
        metadata: Option<Box<dyn Any>>,
    ) {
        let mut subscribers = self.subscribers.lock().unwrap();
        subscribers.push_back(Subscriber {
            observer,
            filter,
            metadata,
        });
    }

    pub fn del(&self, observer: &Weak<dyn Observer<E>>) {
        let mut subscribers = self.subscribers.lock().unwrap();
        subscribers.retain(|subscriber| !Weak::ptr_eq(&subscriber.observer, observer));
    }

    pub fn broadcast(&self, event: &E) {
        let subscribers = self.subscribers.lock().unwrap();
        for subscriber in subscribers.iter() {
            if let Some(filter) = subscriber.filter.as_ref() {
                if !filter.filter(event) {
                    continue;
                }
            }
            let observer = match subscriber.observer.upgrade() {
                // TODO: should remove subscribers whose observers have been freed
                None => return,
                Some(observer) => observer,
            };

            observer.on_event(event, &subscriber.metadata);
        }
    }
}
