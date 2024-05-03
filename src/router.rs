use counted_map::ReassignableCountedMap;

use crate::{exposed::Exposed, receive::{Receive, ReceiverResult}, view::View};

pub trait Route<E>: Receive<E> {
    fn intercept(&mut self, intercept: Box<dyn Route<E, Output = E>>);

    fn take_intercept(&mut self) -> Option<Box<dyn Route<E, Output = E>>>;
}

pub struct Router<E, R: Receive<E>> {
    intercept: Option<Box<dyn Route<E, Output = E>>>,
    reciever: R,
}

impl<E, R: Receive<E>> Router<E, R> {
    pub fn new(reciever: R) -> Self {
        Self {
            intercept: None,
            reciever,
        }
    }

    pub fn with_intercept(intercept: Box<dyn Route<E, Output = E>>, reciever: R) -> Self {
        Self {
            intercept: Some(intercept),
            reciever,
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

    pub fn new_exposed(reciever: R) -> Router<E, Exposed<E, R>> {
        Router {
            intercept: None,
            reciever: Exposed::new(reciever),
        }
    }

    pub fn new_exposed_with_viewers(
        reciever: R,
        viewers: ReassignableCountedMap<usize, Box<dyn View<E>>>,
    ) -> Router<E, Exposed<E, R>> {
        Router {
            intercept: None,
            reciever: Exposed::with_viewers(viewers, reciever),
        }
    }

    pub fn new_exposed_with_intercept(
        intercept: Box<dyn Route<E, Output = E>>,
        reciever: R,
    ) -> Router<E, Exposed<E, R>> {
        Router {
            intercept: Some(intercept),
            reciever: Exposed::new(reciever),
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

    pub fn get_reciever(&self) -> &R {
        &self.reciever
    }

    pub fn get_reciever_mut(&mut self) -> &mut R {
        &mut self.reciever
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
}

impl<E, R: Receive<E>> Receive<E> for Router<E, R> {
    type Output = R::Output;
    fn send(&mut self, event: E) -> ReceiverResult<E, Self::Output> {
        let event = if let Some(ref mut intercept) = self.intercept {
            match intercept.send(event) {
                ReceiverResult::Continue(event) => event,
                ReceiverResult::Stop => return ReceiverResult::Stop,
                ReceiverResult::Delete(event) => {
                    let mut old_intercept = self.take_intercept().unwrap();
                    match old_intercept.take_intercept() {
                        Some(new_intercept) => {
                            self.intercept(new_intercept);
                        },
                        None => self.intercept = None,
                    }
                    event
                },
            }
        } else {
            event
        };

        self.reciever.send(event)
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
