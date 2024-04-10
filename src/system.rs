use crate::update::Updatable;

pub trait System {
    type Dispatcher: System;
    type State: Updatable;
    type Return;

    fn get_state(&mut self) -> &mut Self::State;
    fn execute(
        dispatcher: &mut Self::Dispatcher,
        state: <Self::State as Updatable>::Interface,
    ) -> Self::Return;

    fn post(
        &mut self,
        dispatcher: &mut Self::Dispatcher,
        update: <Self::State as Updatable>::Update,
    ) -> Self::Return {
        let state = self.get_state();
        Self::execute(dispatcher, state.update(update))
    }
}
