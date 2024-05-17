use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use crate::{
    receive::{Receive, ReceiverResult},
    view::{DeleteView, View},
};

#[derive(Clone)]
pub struct ArcLinked<R> {
    pub(super) link: Arc<Mutex<Option<R>>>,
}

impl<R> ArcLinked<R> {
    pub fn get_receiver(&self) -> &Mutex<Option<R>> {
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

impl<R: std::fmt::Debug> std::fmt::Debug for ArcLinked<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{links: {}, receiver: {:?}}}",
            Arc::strong_count(&self.link),
            self.link
        )
    }
}

impl<R: std::fmt::Display> std::fmt::Display for ArcLinked<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.link.try_lock() {
            Ok(lock) => match lock.deref() {
                Some(value) => value.fmt(f),
                None => write!(f, "<deleted>"),
            },
            Err(_) => write!(f, "<locked>"),
        }
    }
}
