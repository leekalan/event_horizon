use crate::update::Updatable;

pub trait System {
    type State: Updatable;
    type Return;

    fn get_state(&mut self) -> &mut Self::State;
    fn execute(state: <Self::State as Updatable>::ValidState) -> Self::Return;

    fn post(&mut self, update: <Self::State as Updatable>::Update) -> Option<Self::Return> {
        let state = self.get_state();
        state.update(update).map(Self::execute)
    }
}