pub mod async_system;
pub mod system;

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use futures::{executor::block_on, join, lock::Mutex};

    use crate::{async_system::AsyncSystem, system::System};

    struct Negate;
    impl AsyncSystem<i32> for Negate {
        type Return = i32;
        async fn async_post(&mut self, update: i32) -> Self::Return {
            -update
        }
    }

    struct Add1;
    impl AsyncSystem<i32> for Add1 {
        type Return = i32;
        async fn async_post(&mut self, update: i32) -> Self::Return {
            update + 1
        }
    }

    struct AddAB;
    impl AsyncSystem<(i32, i32)> for AddAB {
        type Return = i32;
        async fn async_post(&mut self, update: (i32, i32)) -> Self::Return {
            update.0 + update.1
        }
    }

    struct Dispatcher;
    impl AsyncSystem<(i32, i32)> for Dispatcher {
        type Return = i32;
        async fn async_post(&mut self, update: (i32, i32)) -> Self::Return {
            let (mut negate, mut add1) = (Negate, Add1);
            let (a, b) = join!(negate.async_post(update.0), add1.async_post(update.1));
            AddAB.async_post((a, b)).await
        }
    }

    #[test]
    fn async_test() {
        let mut dispatcher = Dispatcher;
        let val = dispatcher.async_post((3, 7));
        println!("{}", block_on(val));
    }

    struct ConcatDispatcher {
        data: ConcatData,
        _system: Rc<Mutex<ConcatSystem>>,
    }
    impl System<(i32, i32)> for ConcatDispatcher {
        type Return = i32;
        fn post(&mut self, update: (i32, i32)) -> Self::Return {
            let a = self.data.post_and_block(update.0);
            let b = self.data.post_and_block(update.1);
            a.unwrap_or_else(|| b.unwrap())
        }
    }

    struct ConcatData {
        internal: Option<i32>,
        concat_system: Rc<Mutex<ConcatSystem>>,
    }
    impl AsyncSystem<i32> for ConcatData {
        type Return = Option<i32>;
        async fn async_post(&mut self, update: i32) -> Self::Return {
            if let Some(val) = self.internal {
                self.internal = None;
                let result = self.concat_system.lock().await.async_post((val, update)).await;
                Some(result)
            } else {
                self.internal = Some(update);
                None
            }
        }
    }

    struct ConcatSystem;
    impl AsyncSystem<(i32, i32)> for ConcatSystem {
        type Return = i32;
        async fn async_post(&mut self, update: (i32, i32)) -> Self::Return {
            update.0 + update.1
        }
    }

    #[test]
    fn concat_test() {
        let concat_system = Rc::new(Mutex::new(ConcatSystem));
        let mut dispatcher = ConcatDispatcher {
            data: ConcatData {
                internal: None,
                concat_system: concat_system.clone(),
            },
            _system: concat_system,
        };
        assert_eq!(dispatcher.post((3, 7)), 10);
        assert_eq!(dispatcher.data.post_and_block(4), None);
        assert_eq!(dispatcher.post((2, 5)), 6);
        assert_eq!(dispatcher.data.post_and_block(3), Some(8));
    }
}
