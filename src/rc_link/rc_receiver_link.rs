use std::{cell::RefCell, rc::Rc};

use crate::receive::Receive;

#[derive(Clone, Default, Debug)]
pub struct RcRecieverLink<R> {
    pub(super) link: Rc<RefCell<Option<R>>>,
}

impl<E, R: Receive<E>> Receive<E> for RcRecieverLink<R> {
    type Output = R::Output;

    fn send(&mut self, event: E) -> Option<Self::Output> {
        self.link.borrow_mut().as_mut()?.send(event)
    }
}
