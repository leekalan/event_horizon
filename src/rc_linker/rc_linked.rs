use std::{cell::RefCell, rc::Rc};

use crate::{
    receive::{Receive, ReceiverResult},
    view::{DeleteView, View},
};

#[derive(Clone, Debug)]
pub struct RcLinked<R> {
    pub(super) link: Rc<RefCell<Option<R>>>,
}

impl<R> RcLinked<R> {
    pub fn get_reciever(&self) -> &RefCell<Option<R>> {
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
