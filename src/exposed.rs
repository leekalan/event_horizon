use counted_map::ReassignableCountedMap;

use crate::{
    receive::{Receive, ReceiverResult},
    view::View,
};

pub struct Exposed<E, R: Receive<E>> {
    viewers: ReassignableCountedMap<usize, Box<dyn View<E>>>,
    receiver: R,
}

impl<E, R: Receive<E>> Exposed<E, R> {
    pub fn new(receiver: R) -> Self {
        Self {
            viewers: ReassignableCountedMap::new(),
            receiver,
        }
    }

    pub fn with_viewers(
        viewers: ReassignableCountedMap<usize, Box<dyn View<E>>>,
        receiver: R,
    ) -> Self {
        Self { viewers, receiver }
    }

    pub fn get_receiver(&self) -> &R {
        &self.receiver
    }

    pub fn get_receiver_mut(&mut self) -> &mut R {
        &mut self.receiver
    }

    pub fn get_viewers(&self) -> &ReassignableCountedMap<usize, Box<dyn View<E>>> {
        &self.viewers
    }

    pub fn add_viewer(
        &mut self,
        other: Box<dyn View<E>>,
    ) -> Result<usize, counted_map::HashMapFull> {
        self.viewers.push(other)
    }

    pub fn box_and_add_viewer(
        &mut self,
        other: impl View<E> + 'static,
    ) -> Result<usize, counted_map::HashMapFull> {
        self.add_viewer(Box::new(other))
    }

    pub fn remove_viewer(&mut self, id: usize) -> Option<Box<dyn View<E>>> {
        self.viewers.remove(id)
    }
}

impl<E, R: Receive<E>> Receive<E> for Exposed<E, R> {
    type Output = R::Output;
    fn send(&mut self, event: E) -> ReceiverResult<E, Self::Output> {
        let mut deleted = Vec::new();

        for (id, viewer) in self.viewers.iter_mut() {
            if viewer.view(&event).is_some() {
                deleted.push(*id);
            }
        }

        for id in deleted {
            self.viewers.remove(id);
        }

        self.receiver.send(event)
    }
}
