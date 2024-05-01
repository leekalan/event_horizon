pub trait Reciever<E> {
    type Output;

    fn recieve(&mut self, event: E) -> Option<Self::Output>;
}