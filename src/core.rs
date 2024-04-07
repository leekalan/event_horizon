use crate::dispatcher::Dispatcher;

pub struct Core<D: Dispatcher> {
    pub dispatcher: D
}