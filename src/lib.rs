pub mod arc_linker;
pub mod exposed;
pub mod rc_linker;
pub mod receive;
pub mod router;
pub mod view;

#[cfg(test)]
mod tests {

    use crate::{
        rc_linker::RcLinker,
        receive::Receive,
        router::{Route, Router},
    };

    #[test]
    fn player_health() {
        struct Player {
            health: i32,
        }
        impl Receive<i32> for Player {
            type Output = ();

            fn send(&mut self, event: i32) -> Option<Self::Output> {
                self.health += event;
                Some(())
            }
        }

        struct Shielded {
            shielded: bool,
        }
        impl Receive<i32> for Shielded {
            type Output = i32;

            fn send(&mut self, event: i32) -> Option<Self::Output> {
                if self.shielded {
                    None
                } else {
                    Some(event)
                }
            }
        }

        let player = Player { health: 100 };
        let shielded = Shielded { shielded: false };

        let player_linker = RcLinker::new(player);
        let shielded_linker = RcLinker::new(shielded);

        let shielded_router = Router::new(shielded_linker.linked());

        let mut router = Router::new(player_linker.linked());

        println!(
            "health: {}, shielded: {}",
            player_linker.borrow().as_ref().unwrap().health,
            shielded_linker.borrow().as_ref().unwrap().shielded,
        );

        router.send(-10);

        println!(
            "health: {}, shielded: {}",
            player_linker.borrow().as_ref().unwrap().health,
            shielded_linker.borrow().as_ref().unwrap().shielded,
        );

        router.intercept(Box::new(shielded_router));
        router.send(-5);

        println!(
            "health: {}, shielded: {}",
            player_linker.borrow().as_ref().unwrap().health,
            shielded_linker.borrow().as_ref().unwrap().shielded,
        );

        shielded_linker.borrow_mut().as_mut().unwrap().shielded = true;
        router.send(-20);

        println!(
            "health: {}, shielded: {}",
            player_linker.borrow().as_ref().unwrap().health,
            shielded_linker.borrow().as_ref().unwrap().shielded,
        );

        shielded_linker.borrow_mut().as_mut().unwrap().shielded = false;
        router.send(-20);

        println!(
            "health: {}, shielded: {}",
            player_linker.borrow().as_ref().unwrap().health,
            shielded_linker.borrow().as_ref().unwrap().shielded,
        );
    }
}
