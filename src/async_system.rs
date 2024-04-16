use futures::executor::block_on;

pub trait AsyncSystem<U> {
    type Return;

    #[allow(async_fn_in_trait)]
    async fn async_post(&mut self, update: U) -> Self::Return;
    fn post_and_block(&mut self, update: U) -> Self::Return {
        block_on(self.async_post(update))
    }
}
