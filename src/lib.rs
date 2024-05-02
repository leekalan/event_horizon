pub mod arc_link;
pub mod exposed;
pub mod rc_link;
pub mod receive;
pub mod router;
pub mod view;

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::{
        rc_link::RcLink,
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

        let player = Rc::new(RefCell::new(Player { health: 100 }));
        let shielded = Rc::new(RefCell::new(Shielded { shielded: false }));

        let player_link = RcLink::new(player.clone());
        let shielded_link = RcLink::new(shielded.clone());

        let shielded_router = Router::new(shielded_link.receiver());

        let mut router = Router::new(player_link.receiver());

        println!(
            "health: {}, shielded: {}",
            player.borrow().health,
            shielded.borrow().shielded
        );

        router.send(-10);

        println!(
            "health: {}, shielded: {}",
            player.borrow().health,
            shielded.borrow().shielded
        );

        router.intercept(Box::new(shielded_router));
        router.send(-5);

        println!(
            "health: {}, shielded: {}",
            player.borrow().health,
            shielded.borrow().shielded
        );

        shielded.borrow_mut().shielded = true;
        router.send(-20);

        println!(
            "health: {}, shielded: {}",
            player.borrow().health,
            shielded.borrow().shielded
        );

        shielded.borrow_mut().shielded = false;
        router.send(-20);

        println!(
            "health: {}, shielded: {}",
            player.borrow().health,
            shielded.borrow().shielded
        );
    }
}
