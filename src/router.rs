use crate::receive::Receive;

pub trait Route<E>: Receive<E> {
    fn intercept(&mut self, intercept: Box<dyn Route<E, Output = E>>);
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

    pub fn get_reciever(&self) -> &R {
        &self.reciever
    }

    pub fn get_intercept(&self) -> Option<&dyn Route<E, Output = E>> {
        self.intercept.as_ref().map(Box::as_ref)
    }
}

impl<E, R: Receive<E>> Receive<E> for Router<E, R> {
    type Output = R::Output;
    fn send(&mut self, event: E) -> Option<Self::Output> {
        let event = if let Some(ref mut intercept) = self.intercept {
            intercept.send(event)?
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
}
