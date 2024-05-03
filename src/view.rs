use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex, RwLock},
};

pub trait View<E> {
    fn view(&mut self, event: &E) -> Option<DeleteView>;
}

pub struct DeleteView;

// Rc + RefCell
impl<E, R: View<E>> View<E> for Rc<RefCell<R>> {
    fn view(&mut self, event: &E) -> Option<DeleteView> {
        self.borrow_mut().view(event)
    }
}

// Arc + Mutex
impl<E, R: View<E>> View<E> for Arc<Mutex<R>> {
    fn view(&mut self, event: &E) -> Option<DeleteView> {
        self.lock().unwrap().view(event)
    }
}

// Arc + RwLock
impl<E, R: View<E>> View<E> for Arc<RwLock<R>> {
    fn view(&mut self, event: &E) -> Option<DeleteView> {
        self.write().unwrap().view(event)
    }
}
