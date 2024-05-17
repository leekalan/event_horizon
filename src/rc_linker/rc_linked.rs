use std::{cell::RefCell, rc::Rc};

use crate::{
    receive::{Receive, ReceiverResult},
    view::{DeleteView, View},
};

#[derive(Clone)]
pub struct RcLinked<R> {
    pub(super) link: Rc<RefCell<Option<R>>>,
}

impl<R> RcLinked<R> {
    pub fn get_receiver(&self) -> &RefCell<Option<R>> {
        &self.link
    }
}

impl<E, R: Receive<E>> Receive<E> for RcLinked<R> {
    type Output = R::Output;

    fn send(&mut self, event: E) -> ReceiverResult<E, Self::Output> {
        match self.link.borrow_mut().as_mut() {
            Some(t0) => t0.send(event),
            None => ReceiverResult::Delete(event),
        }
    }
}

impl<E, R: View<E>> View<E> for RcLinked<R> {
    fn view(&mut self, event: &E) -> Option<DeleteView> {
        match self.link.borrow_mut().as_mut() {
            Some(viewer) => viewer.view(event),
            None => Some(DeleteView),
        }
    }
}

impl<R: std::fmt::Debug> std::fmt::Debug for RcLinked<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{links: {}, receiver: {:?}}}",
            Rc::strong_count(&self.link),
            self.link
        )
    }
}

impl<R: std::fmt::Display> std::fmt::Display for RcLinked<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.link.try_borrow() {
            Ok(r) => match r.as_ref() {
                Some(v) => v.fmt(f),
                None => write!(f, "<deleted>"),
            },
            Err(_) => write!(f, "<borrowed>"),
        }
    }
}
