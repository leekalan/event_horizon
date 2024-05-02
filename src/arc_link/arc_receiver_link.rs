use std::sync::{Arc, Mutex};

use crate::receive::Receive;

#[derive(Clone, Default, Debug)]
pub struct ArcRecieverLink<R> {
    pub(super) link: Arc<Mutex<Option<R>>>,
}

impl<E, R: Receive<E>> Receive<E> for ArcRecieverLink<R> {
    type Output = R::Output;

    fn send(&mut self, event: E) -> Option<Self::Output> {
        self.link.lock().unwrap().as_mut()?.send(event)
    }
}
