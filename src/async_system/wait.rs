use std::{future::Future, sync::{Arc, Mutex}, task::Waker};

#[derive(Debug, Default, Clone)]
pub struct WaitingState {
    flag: bool,
    waker: Option<Waker>,
}
impl WaitingState {
    pub fn new() -> Self {
        WaitingState::default()
    }
    pub fn get_waker(&self) -> Option<Waker> {
        self.waker.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Wait {
    state: Arc<Mutex<WaitingState>>,
}
impl Wait {
    pub fn new(state: Arc<Mutex<WaitingState>>) -> Self {
        Wait {
            state
        }
    }
}
impl Future for Wait {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();
        if state.flag {
            std::task::Poll::Ready(())
        } else {
            state.waker = Some(cx.waker().clone());
            std::task::Poll::Pending
        }
    }
}