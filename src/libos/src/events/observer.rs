use std::any::Any;

use super::Event;
use crate::prelude::*;

/// An obsever receives events from the notifiers to which it has registered.
pub trait Observer<E: Event>: Send + Sync {
    fn on_event(&self, event: &E, metadata: &Option<Box<dyn Any + Send + Sync>>) -> ();
}
