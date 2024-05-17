use crate::view::{DeleteView, View};

use super::{Receive, ReceiverResult};

#[derive(Clone, Copy, Default, Debug)]
pub struct PassReceiver;

impl<E> Receive<E> for PassReceiver {
    type Output = E;
    fn send(&mut self, event: E) -> ReceiverResult<E, Self::Output> {
        ReceiverResult::Continue(event)
    }
}

impl<E> View<E> for PassReceiver {
    fn view(&mut self, _event: &E) -> Option<DeleteView> {
        None
    }
}

impl std::fmt::Display for PassReceiver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PassReceiver")
    }
}
