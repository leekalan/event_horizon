pub trait Updatable: Default {
    type Update;
    type ValidState;

    fn update(&mut self, update: Self::Update) -> Option<Self::ValidState>;
}