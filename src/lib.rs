pub mod arc_linker;
pub mod exposed;
pub mod rc_linker;
pub mod receive;
pub mod router;
pub mod view;

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc, thread};

    use crate::{
        exposed::Exposed,
        rc_linker::RcLinker,
        receive::Receive,
        router::{Route, Router},
        view::View,
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

        let player_linker = RcLinker::new(player.clone());
        let shielded_linker = RcLinker::new(shielded.clone());

        let shielded_router = Router::new(shielded_linker.linked());

        let mut router = Router::new(player_linker.linked());

        assert_eq!(player.borrow().health, 100);
        assert!(!shielded.borrow().shielded);

        router.send(-10);

        assert_eq!(player.borrow().health, 90);
        assert!(!shielded.borrow().shielded);

        router.intercept(Box::new(shielded_router));
        router.send(-5);

        assert_eq!(player.borrow().health, 85);
        assert!(!shielded.borrow().shielded);

        shielded.borrow_mut().shielded = true;
        router.send(-20);

        assert_eq!(player.borrow().health, 85);
        assert!(shielded.borrow().shielded);

        shielded.borrow_mut().shielded = false;
        router.send(-20);

        assert_eq!(player.borrow().health, 65);
        assert!(!shielded.borrow().shielded);
    }

    #[test]
    fn multi_threaded() {
        struct A(i32);
        struct B(i32);

        struct PassAndPrint;
        impl Receive<A> for PassAndPrint {
            type Output = thread::JoinHandle<i32>;

            fn send(&mut self, event: A) -> Option<Self::Output> {
                Some(thread::spawn(move || {
                    thread::sleep(std::time::Duration::from_millis(1000));
                    event.0
                }))
            }
        }
        impl Receive<B> for PassAndPrint {
            type Output = thread::JoinHandle<i32>;

            fn send(&mut self, event: B) -> Option<Self::Output> {
                Some(thread::spawn(move || {
                    thread::sleep(std::time::Duration::from_millis(1000));
                    event.0
                }))
            }
        }

        let mut router_a = Router::new(PassAndPrint);
        let mut router_b = Router::new(PassAndPrint);

        let a = router_a.send(A(1)).unwrap();
        let b = router_b.send(B(2)).unwrap();

        assert_eq!(a.join().unwrap(), 1);
        assert_eq!(b.join().unwrap(), 2);
    }

    #[test]
    fn drop_test() {
        struct Player {
            name: String,
        }
        impl Receive<i32> for Player {
            type Output = ();

            fn send(&mut self, event: i32) -> Option<Self::Output> {
                println!("Player: {} received event: {}", self.name, event);
                Some(())
            }
        }
        impl View<i32> for Player {
            fn view(&mut self, event: &i32) -> Option<crate::view::DeleteView> {
                println!("Player: {} viewed event: {}", self.name, event);
                None
            }
        }

        let player_amy = Player {
            name: "Amy".to_string(),
        };

        let player_bob = Player {
            name: "Bob".to_string(),
        };

        let player_amy_linker = RcLinker::new(player_amy);
        let player_bob_linker = RcLinker::new(player_bob);

        let mut player_amy_router = Router::new(Exposed::new(player_amy_linker.linked()));

        assert!(player_amy_router.send(10).is_some());

        player_amy_router.get_reciever_mut().add_viewer(Box::new(player_bob_linker.linked())).unwrap();

        assert!(player_amy_router.send(20).is_some());

        drop(player_bob_linker);

        assert!(player_amy_router.send(30).is_some());

        drop(player_amy_linker);
        let player_bob = Player {
            name: "Bob".to_string(),
        };
        let player_bob_linker = RcLinker::new(player_bob);
        player_amy_router.get_reciever_mut().add_viewer(Box::new(player_bob_linker.linked())).unwrap();

        assert!(player_amy_router.send(40).is_none());
    }
}
