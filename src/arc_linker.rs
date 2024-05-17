pub mod arc_linked;

use std::{
    ops::Deref,
    sync::{Arc, Mutex, MutexGuard},
};

use self::arc_linked::ArcLinked;

#[derive(Clone)]
pub struct ArcLinker<R> {
    receiver: Arc<Mutex<Option<R>>>,
}

impl<R> ArcLinker<R> {
    pub fn new(receiver: R) -> Self {
        Self {
            receiver: Arc::new(Mutex::new(Some(receiver))),
        }
    }

    pub fn get_receiver(&self) -> &Mutex<Option<R>> {
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

impl<R: Default> Default for ArcLinker<R> {
    fn default() -> Self {
        Self::new(R::default())
    }
}

impl<R: std::fmt::Debug> std::fmt::Debug for ArcLinker<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{links: {}, receiver: {:?}}}",
            Arc::strong_count(&self.receiver),
            self.receiver
        )
    }
}

impl<R: std::fmt::Display> std::fmt::Display for ArcLinker<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.receiver.try_lock() {
            Ok(lock) => match lock.deref() {
                Some(value) => value.fmt(f),
                None => write!(f, "<deleted>"),
            },
            Err(_) => write!(f, "<locked>"),
        }
    }
}
