pub mod wait;
pub mod take;

use std::{future::Future, sync::{Arc, Mutex}};

use crate::update::Updatable;

use self::wait::{Wait, WaitingState};

pub trait AsyncSystem {
    type State: Updatable;
    type Return;

    fn get_state(&mut self) -> &mut Self::State;
    #[allow(async_fn_in_trait)]
    async fn execute(state: <Self::State as Updatable>::ValidState) -> Self::Return;
    fn get_waiting_states(&mut self) -> &mut Vec<Arc<Mutex<WaitingState>>>;

    fn create_wait(&mut self) -> Wait {
        let waiting_states = self.get_waiting_states();
        let new_wait = Arc::new(Mutex::new(WaitingState::default()));
        waiting_states.push(new_wait.clone());
        Wait::new(new_wait)
    }

    fn notify_all(&mut self) {
        let waiting_states = self.get_waiting_states();
        for state in waiting_states.iter() {
            if let Some(waker) = state.lock().unwrap().get_waker() {
                waker.wake();
            }
        }
    }

    fn post(&mut self, update: <Self::State as Updatable>::Update) -> Result<impl Future<Output = Self::Return>, Wait> {
        let state = self.get_state();
        match state.update(update) {
            Some(valid) => Ok(async {
                let val = Self::execute(valid).await;
                self.notify_all();
                val
            }),
            None => Err(self.create_wait()),
        }
    }
}