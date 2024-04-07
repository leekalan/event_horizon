use crate::{core::Core, dispatcher::Dispatcher};

pub trait Process {
    type Core: Dispatcher;

    fn run(&mut self, core: &mut Core<Self::Core>);
}