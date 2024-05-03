use super::{Receive, ReceiverResult};

#[derive(Clone, Copy, Default, Debug)]
pub struct PassReceiver;

impl<E> Receive<E> for PassReceiver {
    type Output = E;
    fn send(&mut self, event: E) -> ReceiverResult<E, Self::Output> {
        ReceiverResult::Continue(event)
    }
}
