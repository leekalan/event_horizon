pub mod arc_linked;

use std::{ops::Deref, sync::{Arc, Mutex, MutexGuard}};

use self::arc_linked::ArcLinked;

#[derive(Debug)]
pub struct ArcLinker<R> {
    receiver: Arc<Mutex<Option<R>>>,
}

impl<R> ArcLinker<R> {
    pub fn new(receiver: R) -> Self {
        Self {
            receiver: Arc::new(Mutex::new(Some(receiver))),
        }
    }

    pub fn get_reciever(&self) -> &Mutex<Option<R>> {
        self
    }

    pub fn lock(&self) -> MutexGuard<Option<R>> {
        self.receiver.lock().unwrap()
    }

    pub fn linked(&self) -> ArcLinked<R> {
        ArcLinked {
            link: self.receiver.clone(),
        }
    }
}

impl<R> Deref for ArcLinker<R> {
    type Target = Mutex<Option<R>>;

    fn deref(&self) -> &Self::Target {
        &self.receiver
    }
}

impl<R> Drop for ArcLinker<R> {
    fn drop(&mut self) {
        *self.receiver.lock().unwrap() = None;
    }
}
