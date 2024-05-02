pub mod arc_receiver_link;
pub mod arc_viewer_link;

use std::sync::{Arc, Mutex};

use self::{arc_receiver_link::ArcRecieverLink, arc_viewer_link::ArcViewerLink};

#[derive(Clone, Default, Debug)]
pub struct ArcLink<R> {
    receiver: Arc<Mutex<Option<R>>>,
}

impl<R> ArcLink<R> {
    pub fn new(receiver: R) -> Self {
        Self {
            receiver: Arc::new(Mutex::new(Some(receiver))),
        }
    }

    pub fn receiver(&self) -> ArcRecieverLink<R> {
        ArcRecieverLink {
            link: self.receiver.clone(),
        }
    }

    pub fn viewer(&self) -> ArcViewerLink<R> {
        ArcViewerLink {
            link: self.receiver.clone(),
        }
    }
}

impl<R> Drop for ArcLink<R> {
    fn drop(&mut self) {
        *self.receiver.lock().unwrap() = None;
    }
}
