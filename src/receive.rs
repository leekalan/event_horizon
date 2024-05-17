use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex, RwLock},
};

pub mod pass_receiver;

pub trait Receive<E> {
    type Output;

    fn send(&mut self, event: E) -> ReceiverResult<E, Self::Output>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReceiverResult<E, T> {
    Continue(T),
    Stop,
    Delete(E),
}
impl<E, T> ReceiverResult<E, T> {
    pub fn is_continue(&self) -> bool {
        matches!(self, ReceiverResult::Continue(_))
    }
    pub fn is_stop(&self) -> bool {
        matches!(self, ReceiverResult::Stop)
    }
    pub fn is_delete(&self) -> bool {
        matches!(self, ReceiverResult::Delete(_))
    }

    pub fn unwrap_continue(self) -> T {
        match self {
            ReceiverResult::Continue(t) => t,
            _ => panic!(),
        }
    }
    pub fn unwrap_delete(self) -> E {
        match self {
            ReceiverResult::Delete(e) => e,
            _ => panic!(),
        }
    }
}

// Rc + RefCell
impl<E, R: Receive<E>> Receive<E> for Rc<RefCell<R>> {
    type Output = R::Output;

    fn send(&mut self, event: E) -> ReceiverResult<E, Self::Output> {
        self.borrow_mut().send(event)
    }
}

// Rc + Mutex
impl<E, R: Receive<E>> Receive<E> for Rc<Mutex<R>> {
    type Output = R::Output;

    fn send(&mut self, event: E) -> ReceiverResult<E, Self::Output> {
        self.lock().unwrap().send(event)
    }
}

// Rc + RwLock
impl<E, R: Receive<E>> Receive<E> for Rc<RwLock<R>> {
    type Output = R::Output;

    fn send(&mut self, event: E) -> ReceiverResult<E, Self::Output> {
        self.write().unwrap().send(event)
    }
}

// Arc + Mutex
impl<E, R: Receive<E>> Receive<E> for Arc<Mutex<R>> {
    type Output = R::Output;

    fn send(&mut self, event: E) -> ReceiverResult<E, Self::Output> {
        self.lock().unwrap().send(event)
    }
}

// Arc + RwLock
impl<E, R: Receive<E>> Receive<E> for Arc<RwLock<R>> {
    type Output = R::Output;

    fn send(&mut self, event: E) -> ReceiverResult<E, Self::Output> {
        self.write().unwrap().send(event)
    }
}
