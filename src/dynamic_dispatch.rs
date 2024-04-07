use crate::{dispatcher::Dispatcher, process::Process};

pub struct DynamicDispatch<E, D: Dispatcher> {
    processes: Vec<Box<dyn Process<Core = D>>>,
    buffer: Vec<E>,
}
impl<E, D: Dispatcher> DynamicDispatch<E, D> {
    fn post(&mut self, event: E) {
        self.buffer.push(event);
    }
    fn get_events(&self) -> impl Iterator<Item = &E> {
        self.buffer.iter()
    }
    fn find_event(&self, closure: impl FnMut(&E) -> bool) -> Option<&E> {
        let mut closure = closure;
        self.buffer.iter().find(|s| closure(*s))
    }
}
impl<E, D: Dispatcher> Dispatcher for DynamicDispatch<E, D> {
    type Core = D;

    fn get_processes(&mut self) -> impl Iterator<Item = &mut dyn Process<Core = Self::Core>> {
        self.processes.iter_mut().map(|x| &mut **x as &mut dyn Process<Core = Self::Core>)
    }
}
impl<E, D: Dispatcher> Process for DynamicDispatch<E, D> {
    type Core = D;

    fn run(&mut self, core: &mut crate::core::Core<Self::Core>) {
        self.dispatch(core)
    }
}