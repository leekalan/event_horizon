use super::Receive;

#[derive(Clone, Copy, Default, Debug)]
pub struct PassReceiver;

impl<E> Receive<E> for PassReceiver {
    type Output = E;
    fn send(&mut self, event: E) -> Option<Self::Output> {
        Some(event)
    }
}
