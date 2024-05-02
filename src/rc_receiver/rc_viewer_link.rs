use std::{cell::RefCell, rc::Rc};

use crate::view::{DeleteView, View};

#[derive(Clone, Default, Debug)]
pub struct RcViewerLink<R> {
    pub(super) link: Rc<RefCell<Option<R>>>,
}

impl<E, R: View<E>> View<E> for RcViewerLink<R> {
    fn view(&mut self, event: &E) -> Option<DeleteView> {
        match self.link.borrow_mut().as_mut() {
            Some(viewer) => viewer.view(event),
            None => Some(DeleteView),
        }
    }
}
