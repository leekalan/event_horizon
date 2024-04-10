use crate::update::Updatable;

pub trait AsyncSystem {
    type Dispatcher: AsyncSystem;
    type State: Updatable;
    type Return;

    fn get_state(&mut self) -> &mut Self::State;
    #[allow(async_fn_in_trait)]
    async fn execute(
        dispatcher: &mut Self::Dispatcher,
        state: <Self::State as Updatable>::Interface,
    ) -> Self::Return;

    #[allow(async_fn_in_trait)]
    async fn post(
        &mut self,
        dispatcher: &mut Self::Dispatcher,
        update: <Self::State as Updatable>::Update,
    ) -> Self::Return {
        let state = self.get_state();
        Self::execute(dispatcher, state.update(update)).await
    }
}
