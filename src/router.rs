use counted_map::ReassignableCountedMap;

use crate::{
    exposed::Exposed,
    receive::{Receive, ReceiverResult},
    view::View,
};

pub trait Route<E>: Receive<E> {
    fn intercept(&mut self, intercept: Box<dyn Route<E, Output = E>>);

    fn take_intercept(&mut self) -> Option<Box<dyn Route<E, Output = E>>>;

    fn intercept_at_root(&mut self, intercept: Box<dyn Route<E, Output = E>>) {
        let old_intercept = self.take_intercept();
        match old_intercept {
            Some(r) => {
                self.intercept(intercept);
                self.intercept(r);
            }
            None => self.intercept(intercept),
        }
    }
}

pub struct Router<E, R: Receive<E>> {
    intercept: Option<Box<dyn Route<E, Output = E>>>,
    receiver: R,
}

impl<E, R: Receive<E>> Router<E, R> {
    pub fn new(receiver: R) -> Self {
        Self {
            intercept: None,
            receiver,
        }
    }

    pub fn with_intercept(intercept: Box<dyn Route<E, Output = E>>, receiver: R) -> Self {
        Self {
            intercept: Some(intercept),
            receiver,
        }
    }

    pub fn with_intercept_from_receiver(
        intercept: impl Receive<E, Output = E> + 'static,
        receiver: R,
    ) -> Self
    where
        E: 'static,
    {
        Self::with_intercept(Box::new(Router::new(intercept)), receiver)
    }

    pub fn new_exposed(receiver: R) -> Router<E, Exposed<E, R>> {
        Router {
            intercept: None,
            receiver: Exposed::new(receiver),
        }
    }

    pub fn new_exposed_with_viewers(
        receiver: R,
        viewers: ReassignableCountedMap<usize, Box<dyn View<E>>>,
    ) -> Router<E, Exposed<E, R>> {
        Router {
            intercept: None,
            receiver: Exposed::with_viewers(viewers, receiver),
        }
    }

    pub fn new_exposed_with_intercept(
        intercept: Box<dyn Route<E, Output = E>>,
        receiver: R,
    ) -> Router<E, Exposed<E, R>> {
        Router {
            intercept: Some(intercept),
            receiver: Exposed::new(receiver),
        }
    }

    pub fn new_exposed_with_intercept_from_receiver(
        intercept: impl Receive<E, Output = E> + 'static,
        receiver: R,
    ) -> Router<E, Exposed<E, R>>
    where
        E: 'static,
    {
        Router::new_exposed_with_intercept(Box::new(Router::new(intercept)), receiver)
    }

    pub fn get_receiver(&self) -> &R {
        &self.receiver
    }

    pub fn get_receiver_mut(&mut self) -> &mut R {
        &mut self.receiver
    }

    pub fn get_intercept(&self) -> Option<&dyn Route<E, Output = E>> {
        self.intercept.as_ref().map(Box::as_ref)
    }

    pub fn intercept_from_receiver(&mut self, intercept: impl Receive<E, Output = E> + 'static)
    where
        E: 'static,
    {
        let intercept = Box::new(Router::new(intercept));
        self.intercept(intercept);
    }

    pub fn intercept_at_root_from_receiver(
        &mut self,
        intercept: impl Receive<E, Output = E> + 'static,
    ) where
        E: 'static,
    {
        let intercept = Box::new(Router::new(intercept));
        self.intercept_at_root(intercept);
    }

    pub fn delete_top_intercept(&mut self) -> Option<Box<dyn Route<E, Output = E>>> {
        let mut old_intercept = self.take_intercept();
        if let Some(ref mut intercept) = old_intercept {
            self.intercept = intercept.take_intercept();
        }
        old_intercept
    }
}

impl<E, R: Receive<E>> Receive<E> for Router<E, R> {
    type Output = R::Output;
    fn send(&mut self, event: E) -> ReceiverResult<E, Self::Output> {
        let event = if let Some(ref mut intercept) = self.intercept {
            match intercept.send(event) {
                ReceiverResult::Continue(event) => event,
                ReceiverResult::Stop => return ReceiverResult::Stop,
                ReceiverResult::Delete(event) => {
                    self.delete_top_intercept().unwrap();
                    event
                }
            }
        } else {
            event
        };

        self.receiver.send(event)
    }
}

impl<E, R: Receive<E>> Route<E> for Router<E, R> {
    fn intercept(&mut self, intercept: Box<dyn Route<E, Output = E>>) {
        match self.intercept {
            Some(ref mut child) => child.intercept(intercept),
            None => self.intercept = Some(intercept),
        }
    }

    fn take_intercept(&mut self) -> Option<Box<dyn Route<E, Output = E>>> {
        self.intercept.take()
    }
}

impl<E, R: Receive<E> + Default> Default for Router<E, R> {
    fn default() -> Self {
        Self::new(R::default())
    }
}

impl<E, R: Receive<E> + std::fmt::Debug> std::fmt::Debug for Router<E, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        match self.intercept {
            Some(_) => write!(f, "intercepted, ")?,
            None => write!(f, "no intercept, ")?,
        }
        write!(f, "receiver: {:?}}}", self.receiver)
    }
}

impl<E, R: Receive<E> + std::fmt::Display> std::fmt::Display for Router<E, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.receiver.fmt(f)
    }
}
