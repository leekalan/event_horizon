pub trait Updatable: Default {
    type Update;
    type Interface;

    fn update(&mut self, update: Self::Update) -> Self::Interface;
}
