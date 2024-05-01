use counted_map::{CountedMap, UniqueId};

use crate::{observer::Observer, reciever::Reciever};

pub struct Channel<E, O> {
    intercept: Option<Box<Channel<E, E>>>,
    observers: CountedMap<UniqueId, Box<dyn Observer<E>>>,
    reciever: Option<Box<dyn Reciever<E, Output = O>>>,
}

impl<E> Default for Channel<E, ()> {
    fn default() -> Self {
        Self {
            intercept: None,
            observers: CountedMap::new(),
            reciever: None,
        }
    }
}

impl <E, O> Channel<E, O> {
    pub fn new(reciever: Box<dyn Reciever<E, Output = O>>) -> Self {
        Self {
            intercept: None,
            observers: CountedMap::new(),
            reciever: Some(reciever),
        }
    }

    pub fn send(&mut self, event: E) -> Option<O> {
        let event = match self.intercept {
            Some(ref mut intercept) => intercept.send(event)?,
            None => event,
        };

        for observer in self.observers.values_mut() {
            observer.observe(&event);
        }

        match self.reciever {
            Some(ref mut reciever) => reciever.recieve(event),
            None => None,
        }
    }
}