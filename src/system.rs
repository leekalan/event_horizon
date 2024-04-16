pub trait System<U> {
    type Return;

    fn post(&mut self, update: U) -> Self::Return;
}