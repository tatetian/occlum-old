use std::any::Any;

use super::Event;
use crate::prelude::*;

pub trait Observer<E: Event> {
    fn on_event(&self, event: &E, metadata: &Option<Box<dyn Any>>) -> ();
}
