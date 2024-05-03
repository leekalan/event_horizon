//! A library for creating event-based applications
//!
//! ## Overview
//!
//! The library consists of 2 traits:
//! - [`Receive`][`receive::Receive`]: a generic interface for sending events
//! - [`View`][`view::View`]: a generic interface for viewing events
//! 
//! Viewer return;
//! - [`DeleteView`][`view::DeleteView`]: flag to delete the viewer
//! 
//! Receiver return, [`ReceiverResult`][`receive::ReceiverResult`]:
//! - [`Continue`][`receive::ReceiverResult::Continue`]: continue processing the output as normal (like [`Some`])
//! - [`Stop`][`receive::ReceiverResult::Stop`]: stop processing the output (like [`None`]`)
//! - [`Delete`][`receive::ReceiverResult::Delete`]: gives back the event with the flag that the receiver should be
//! deleted, this is specifically for communication with routers so that intercepts can be cleanly destructed while
//! letting the event pass through
//!
//! The different ways to store receivers and viewers are:
//! - [`RcLinker`][`rc_linker::RcLinker`] and [`ArcLinker`][`arc_linker::ArcLinker`]:
//! a mutexed container for a type that will invalidate any instances of [`RcLinked`] or [`ArcLinked`]
//! ready for deletion when dropped, cleaning up any dangling references.
//! - [`Exposed`][`exposed::Exposed`]: a container for a receiver that allows multiple [`View`][`view::View`]s to be prepended
//! - [`Router`][`router::Router`]: a container for a receiver that allows another router to intercept the event at the beginning,
//! by repeating the intercept function it will be delegated to lower routers, allowing a level of abstraction where an intercept
//! does what is expected without breaking the rest of the route.

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
        rc_linker::RcLinker,
        receive::{Receive, ReceiverResult},
        router::Router,
        view::View,
    };

    #[test]
    fn player_health() {
        struct Player {
            health: i32,
        }
        impl Receive<i32> for Player {
            type Output = ();

            fn send(&mut self, event: i32) -> ReceiverResult<i32, Self::Output> {
                self.health += event;
                ReceiverResult::Continue(())
            }
        }

        struct Shielded {
            shielded: bool,
        }
        impl Receive<i32> for Shielded {
            type Output = i32;

            fn send(&mut self, event: i32) -> ReceiverResult<i32, Self::Output> {
                if self.shielded {
                    ReceiverResult::Stop
                } else {
                    ReceiverResult::Continue(event)
                }
            }
        }

        let player = Rc::new(RefCell::new(Player { health: 100 }));
        let shielded = Rc::new(RefCell::new(Shielded { shielded: false }));

        let player_linker = RcLinker::new(player.clone());
        let shielded_linker = RcLinker::new(shielded.clone());

        let mut router = Router::new(player_linker.linked());

        assert_eq!(player.borrow().health, 100);
        assert!(!shielded.borrow().shielded);

        router.send(-10);

        assert_eq!(player.borrow().health, 90);
        assert!(!shielded.borrow().shielded);

        router.intercept_from_receiver(shielded_linker.linked());
        router.send(-5);

        assert_eq!(player.borrow().health, 85);
        assert!(!shielded.borrow().shielded);

        shielded.borrow_mut().shielded = true;
        router.send(-20);

        assert_eq!(player.borrow().health, 85);
        assert!(shielded.borrow().shielded);

        drop(shielded_linker);
        router.send(-20);

        assert_eq!(player.borrow().health, 65);
    }

    #[test]
    fn multi_threaded() {
        struct A(i32);
        struct B(i32);

        struct PassAndPrint;
        impl Receive<A> for PassAndPrint {
            type Output = thread::JoinHandle<i32>;

            fn send(&mut self, event: A) -> ReceiverResult<A, Self::Output> {
                ReceiverResult::Continue(thread::spawn(move || {
                    thread::sleep(std::time::Duration::from_millis(1000));
                    event.0
                }))
            }
        }
        impl Receive<B> for PassAndPrint {
            type Output = thread::JoinHandle<i32>;

            fn send(&mut self, event: B) -> ReceiverResult<B, Self::Output> {
                ReceiverResult::Continue(thread::spawn(move || {
                    thread::sleep(std::time::Duration::from_millis(1000));
                    event.0
                }))
            }
        }

        let mut router_a = Router::new(PassAndPrint);
        let mut router_b = Router::new(PassAndPrint);

        let a = router_a.send(A(1)).unwrap_continue();
        let b = router_b.send(B(2)).unwrap_continue();

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

            fn send(&mut self, event: i32) -> ReceiverResult<i32, Self::Output> {
                println!("Player: {} received event: {}", self.name, event);
                ReceiverResult::Continue(())
            }
        }
        impl View<i32> for Player {
            fn view(&mut self, event: &i32) -> Option<crate::view::DeleteView> {
                println!("Player: {} viewed event: {}", self.name, event);
                None
            }
        }

        let player_amy_linker = RcLinker::new(Player {
            name: "Amy".to_string(),
        });

        let player_bob_linker = RcLinker::new(Player {
            name: "Bob".to_string(),
        });

        let mut player_amy_router = Router::new_exposed(player_amy_linker.linked());

        assert!(player_amy_router.send(10).is_continue());

        player_amy_router
            .get_reciever_mut()
            .box_and_add_viewer(player_bob_linker.linked())
            .unwrap();

        assert!(player_amy_router.send(20).is_continue());

        drop(player_bob_linker);

        assert!(player_amy_router.send(30).is_continue());

        drop(player_amy_linker);

        let player_collin_linker = RcLinker::new(Player {
            name: "Collin".to_string(),
        });
        player_amy_router
            .get_reciever_mut()
            .box_and_add_viewer(player_collin_linker.linked())
            .unwrap();

        assert!(player_amy_router.send(40).is_delete());

        drop(player_collin_linker);
    }

    #[test]
    fn nested_intercepts() {
        struct Reader {
            name: String,
        }
        impl Receive<i32> for Reader {
            type Output = i32;
            
            fn send(&mut self, event: i32) -> ReceiverResult<i32, Self::Output> {
                println!("Reader: {} received event: {}", self.name, event);
                ReceiverResult::Continue(event)
            }
        }

        let reader_a = RcLinker::new(Reader {
            name: "A".to_string(),
        });
        let reader_b = RcLinker::new(Reader {
            name: "B".to_string(),
        });
        let reader_c = RcLinker::new(Reader {
            name: "C".to_string(),
        });

        let mut router = Router::new(reader_a.linked());

        assert!(router.send(10).is_continue());

        router.intercept_from_receiver(reader_c.linked());

        assert!(router.send(20).is_continue());

        router.intercept_at_root_from_receiver(reader_b.linked());

        assert!(router.send(30).is_continue());

        drop(reader_b);

        assert!(router.send(40).is_continue());

        drop(reader_c);

        assert!(router.send(50).is_continue());

        drop(reader_a);

        assert!(router.send(60).is_delete());
    }
}
