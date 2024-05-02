use counted_map::ReassignableCountedMap;

use crate::{receive::Receive, view::View};

pub struct Exposed<E, R: Receive<E>> {
    viewers: ReassignableCountedMap<usize, Box<dyn View<E>>>,
    reciever: R,
}

impl<E, R: Receive<E>> Exposed<E, R> {
    pub fn new(reciever: R) -> Box<Self> {
        Box::new(Self {
            viewers: ReassignableCountedMap::new(),
            reciever,
        })
    }
}

impl<E, R: Receive<E>> Receive<E> for Exposed<E, R> {
    type Output = R::Output;
    fn send(&mut self, event: E) -> Option<Self::Output> {
        let mut deleted = Vec::new();

        for (id, viewer) in self.viewers.iter_mut() {
            if viewer.view(&event).is_none() {
                deleted.push(*id);
            }
        }

        for id in deleted {
            self.viewers.remove(id);
        }

        self.reciever.send(event)
    }
}
