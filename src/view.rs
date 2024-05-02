pub trait View<E> {
    fn view(&mut self, event: &E) -> Option<DeleteView>;
}

pub struct DeleteView;
