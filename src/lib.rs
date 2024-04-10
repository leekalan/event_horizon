pub mod async_system;
pub mod system;
pub mod update;

#[cfg(test)]
mod tests {
    use std::alloc::System;

    use crate::update::Updatable;

    #[test]
    fn adder_test() {
        #[derive(Default)]
        struct AdderUpdate(Option<i32>);
        impl Updatable for AdderUpdate {
            type Update = i32;
            type Interface = Option<AdderPair>;
        
            fn update(&mut self, update: Self::Update) -> Self::Interface {
                match self.0 {
                    Some(val) => {
                        let output = AdderPair(val, update);
                        self.0 = None;
                        Some(output)
                    },
                    None => {
                        self.0 = Some(update);
                        None
                    },
                }
            }
        }
        struct AdderPair(i32, i32);

        struct Adder {

        }
        impl System for Adder {
            
        }
    }
}
