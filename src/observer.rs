pub trait Observer<E> {
    fn observe(&mut self, event: &E);
}