pub mod rc_receiver_link;
pub mod rc_viewer_link;

use std::{cell::RefCell, rc::Rc};

use self::rc_receiver_link::RcRecieverLink;

#[derive(Clone, Default, Debug)]
pub struct RcReciever<R> {
    receiver: Rc<RefCell<Option<R>>>,
}

impl<R> RcReciever<R> {
    pub fn new(receiver: R) -> Self {
        Self {
            receiver: Rc::new(RefCell::new(Some(receiver))),
        }
    }

    pub fn create_link(&self) -> RcRecieverLink<R> {
        RcRecieverLink {
            link: self.receiver.clone(),
        }
    }
}

impl<R> Drop for RcReciever<R> {
    fn drop(&mut self) {
        *self.receiver.borrow_mut() = None;
    }
}
