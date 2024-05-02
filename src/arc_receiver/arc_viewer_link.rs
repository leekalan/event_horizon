use std::sync::{Arc, Mutex};

use crate::view::{DeleteView, View};

#[derive(Clone, Default, Debug)]
pub struct ArcViewerLink<R> {
    pub(super) link: Arc<Mutex<Option<R>>>,
}

impl<E, R: View<E>> View<E> for ArcViewerLink<R> {
    fn view(&mut self, event: &E) -> Option<DeleteView> {
        match self.link.lock().unwrap().as_mut() {
            Some(viewer) => viewer.view(event),
            None => Some(DeleteView),
        }
    }
}
