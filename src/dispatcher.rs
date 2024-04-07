use crate::{core::Core, process::Process};

pub trait Dispatcher {
    type Core: Dispatcher;

    fn dispatch(&mut self, core: &mut Core<Self::Core>) {
        let processes = self.get_processes();
        for process in processes {
            process.run(core);
        }
    }

    fn get_processes(&mut self) -> impl Iterator<Item = &mut dyn Process<Core = Self::Core>>;
}