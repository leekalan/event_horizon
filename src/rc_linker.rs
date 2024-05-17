pub mod rc_linked;

use std::{
    cell::{Ref, RefCell, RefMut},
    ops::Deref,
    rc::Rc,
};

use self::rc_linked::RcLinked;

#[derive(PartialEq, Eq, Clone)]
pub struct RcLinker<R> {
    receiver: Rc<RefCell<Option<R>>>,
}

impl<R> RcLinker<R> {
    pub fn new(receiver: R) -> Self {
        Self {
            receiver: Rc::new(RefCell::new(Some(receiver))),
        }
    }

    pub fn get_receiver(&self) -> &RefCell<Option<R>> {
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

impl<R: Default> Default for RcLinker<R> {
    fn default() -> Self {
        Self::new(R::default())
    }
}

impl<R: std::fmt::Debug> std::fmt::Debug for RcLinker<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{links: {}, receiver: {:?}}}",
            Rc::strong_count(&self.receiver),
            self.receiver
        )
    }
}

impl<R: std::fmt::Display> std::fmt::Display for RcLinker<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.receiver.try_borrow() {
            Ok(r) => match r.as_ref() {
                Some(v) => v.fmt(f),
                None => write!(f, "<deleted>"),
            },
            Err(_) => write!(f, "<borrowed>"),
        }
    }
}
