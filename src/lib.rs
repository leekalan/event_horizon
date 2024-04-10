pub mod system;
pub mod async_system;
pub mod update;

#[cfg(test)]
mod tests {
    use crate::{async_system::{wait::WaitingState, AsyncSystem}, system::System, update::Updatable};

    #[repr(transparent)]
    #[derive(Default)]
    struct NumPairState(Option<i32>);
    impl Updatable for NumPairState {
        type Update = i32;
        type ValidState = (i32, i32);
    
        fn update(&mut self, update: Self::Update) -> Option<Self::ValidState> {
            match self.0 {
                Some(num) => {
                    self.0 = None;
                    Some((num, update))
                },
                None => {
                    self.0 = Some(update);
                    None
                },
            }
        }
    }

    #[derive(Default)]
    struct NumPairSystem {
        state: NumPairState,
        wakers: Vec<std::sync::Arc<std::sync::Mutex<WaitingState>>>,
    }
    impl System for NumPairSystem {
        type State = NumPairState;
        type Return = i32;
    
        fn get_state(&mut self) -> &mut Self::State {
            &mut self.state
        }
    
        fn execute(state: <Self::State as Updatable>::ValidState) -> Self::Return {
            state.0 + state.1
        }
    }

    #[test]
    fn test() {
        let mut system = NumPairSystem::default();
        assert_eq!(system.post(2), None);
        assert_eq!(system.post(3), Some(5));
    }
}
