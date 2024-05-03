use std::sync::{Arc, Mutex};

use crate::{
    receive::{Receive, ReceiverResult},
    view::{DeleteView, View},
};

#[derive(Clone, Debug)]
pub struct ArcLinked<R> {
    pub(super) link: Arc<Mutex<Option<R>>>,
}

impl<R> ArcLinked<R> {
    pub fn get_reciever(&self) -> &Mutex<Option<R>> {
        self.link.as_ref()
    }
}

impl<E, R: Receive<E>> Receive<E> for ArcLinked<R> {
    type Output = R::Output;

    fn send(&mut self, event: E) -> ReceiverResult<E, Self::Output> {
        match self.link.lock().unwrap().as_mut() {
            Some(t0) => t0.send(event),
            None => ReceiverResult::Delete(event),
        }
    }
}

impl<E, R: View<E>> View<E> for ArcLinked<R> {
    fn view(&mut self, event: &E) -> Option<DeleteView> {
        match self.link.lock().unwrap().as_mut() {
            Some(viewer) => viewer.view(event),
            None => Some(DeleteView),
        }
    }
}
