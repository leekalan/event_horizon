use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex, RwLock},
};

pub mod pass_receiver;
pub mod void_receiver;

pub trait Receive<E> {
    type Output;

    fn send(&mut self, event: E) -> Option<Self::Output>;
}

// Rc + RefCell
impl<E, R: Receive<E>> Receive<E> for Rc<RefCell<R>> {
    type Output = R::Output;

    fn send(&mut self, event: E) -> Option<Self::Output> {
        self.borrow_mut().send(event)
    }
}

// Arc + Mutex
impl<E, R: Receive<E>> Receive<E> for Arc<Mutex<R>> {
    type Output = R::Output;

    fn send(&mut self, event: E) -> Option<Self::Output> {
        self.lock().unwrap().send(event)
    }
}

// Arc + RwLock
impl<E, R: Receive<E>> Receive<E> for Arc<RwLock<R>> {
    type Output = R::Output;

    fn send(&mut self, event: E) -> Option<Self::Output> {
        self.write().unwrap().send(event)
    }
}
