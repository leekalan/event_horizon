pub mod rc_linked;

use std::{
    cell::{Ref, RefCell, RefMut}, ops::Deref, rc::Rc
};

use self::rc_linked::RcLinked;

#[derive(Debug)]
pub struct RcLinker<R> {
    receiver: Rc<RefCell<Option<R>>>,
}

impl<R> RcLinker<R> {
    pub fn new(receiver: R) -> Self {
        Self {
            receiver: Rc::new(RefCell::new(Some(receiver))),
        }
    }

    pub fn default_instance() -> Self
    where
        R: Default,
    {
        Self {
            receiver: Rc::new(RefCell::new(Some(Default::default()))),
        }
    }

    pub fn get_reciever(&self) -> &RefCell<Option<R>> {
        self
    }

    pub fn borrow(&self) -> Ref<Option<R>> {
        self.receiver.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<Option<R>> {
        self.receiver.borrow_mut()
    }

    pub fn linked(&self) -> RcLinked<R> {
        RcLinked {
            link: self.receiver.clone(),
        }
    }
}

impl<R> Deref for RcLinker<R> {
    type Target = RefCell<Option<R>>;

    fn deref(&self) -> &Self::Target {
        &self.receiver
    }
}

impl<R> Drop for RcLinker<R> {
    fn drop(&mut self) {
        *self.receiver.borrow_mut() = None;
    }
}
