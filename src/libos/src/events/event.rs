pub trait Event: Copy + Clone + Send + 'static {}

pub trait EventFilter<E: Event>: Send + 'static {
    fn filter(&self, event: &E) -> bool;
}
