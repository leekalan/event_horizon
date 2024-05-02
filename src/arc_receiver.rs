pub mod arc_receiver_link;
pub mod arc_viewer_link;

use std::sync::{Arc, Mutex};

use self::arc_receiver_link::ArcRecieverLink;

#[derive(Clone, Default, Debug)]
pub struct ArcReciever<R> {
    receiver: Arc<Mutex<Option<R>>>,
}

impl<R> ArcReciever<R> {
    pub fn new(receiver: R) -> Self {
        Self {
            receiver: Arc::new(Mutex::new(Some(receiver))),
        }
    }

    pub fn create_link(&self) -> ArcRecieverLink<R> {
        ArcRecieverLink {
            link: self.receiver.clone(),
        }
    }
}

impl<R> Drop for ArcReciever<R> {
    fn drop(&mut self) {
        *self.receiver.lock().unwrap() = None;
    }
}
