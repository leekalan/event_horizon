use super::Receive;

#[derive(Clone, Copy, Default, Debug)]
pub struct VoidReceiver;

impl<E> Receive<E> for VoidReceiver {
    type Output = ();
    fn send(&mut self, _event: E) -> Option<Self::Output> {
        None
    }
}
