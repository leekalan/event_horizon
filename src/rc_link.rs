pub mod rc_receiver_link;
pub mod rc_viewer_link;

use std::{cell::RefCell, rc::Rc};

use self::{rc_receiver_link::RcRecieverLink, rc_viewer_link::RcViewerLink};

#[derive(Clone, Default, Debug)]
pub struct RcLink<R> {
    receiver: Rc<RefCell<Option<R>>>,
}

impl<R> RcLink<R> {
    pub fn new(receiver: R) -> Self {
        Self {
            receiver: Rc::new(RefCell::new(Some(receiver))),
        }
    }

    pub fn receiver(&self) -> RcRecieverLink<R> {
        RcRecieverLink {
            link: self.receiver.clone(),
        }
    }

    pub fn viewer(&self) -> RcViewerLink<R> {
        RcViewerLink {
            link: self.receiver.clone(),
        }
    }
}

impl<R> Drop for RcLink<R> {
    fn drop(&mut self) {
        *self.receiver.borrow_mut() = None;
    }
}
