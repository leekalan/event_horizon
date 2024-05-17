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
//! a smart pointer that will mark any instances of [`RcLinked`][`rc_linker::rc_linked::RcLinked`] or
//! [`ArcLinked`][`arc_linker::arc_linked::ArcLinked`] ready for deletion when dropped, cleaning up any
//! dangling references.
//! - [`Exposed`][`exposed::Exposed`]: a container for a receiver that allows multiple [`View`][`view::View`]ers to be
//! prepended
//! - [`Router`][`router::Router`]: a container for a receiver that allows another router to intercept the event at
//! the beginning, by repeating the intercept function it will be delegated to lower routers, allowing a level of
//! abstraction where an intercept does what is expected without breaking the rest of the router.
//!
//! ## Aproach
//!
//! The receivers function as a sort of lazy garbage collector.
//!
//! When receivers flags that they wish to be deleted ([`Delete`][`receive::ReceiverResult::Delete`]), it should
//! be expected that everything occuring before the introduction of the flag was ran, including viewers and intercepters.
//!
//! **This mean if *[`Delete`][`receive::ReceiverResult::Delete`]* is received, it is expected that *all prior systems
//! have ran* and responsibilty falls upon the receiver to *continue the event propgation with minimal interuptions*.**
//!
//! The exception to this is [`Stop`][`receive::ReceiverResult::Stop`], which marks that a decision has been made to exit
//! the event propgation.
//!
//! **This means if *[`Stop`][`receive::ReceiverResult::Stop`]* is received, it is expected that *not all prior systems
//! have ran* and responsibilty falls upon the receiver to *exit the event propgation with minimal impact*.**
//!

pub mod arc_linker;
pub mod exposed;
pub mod multi_exposed;
pub mod multi_router;
pub mod rc_linker;
pub mod receive;
pub mod router;
pub mod view;

pub use crate as event_horizon;
pub use counted_map;

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc, thread};

    use crate::{
        multi_exposed::MultiExpose,
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
            .get_receiver_mut()
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
            .get_receiver_mut()
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

    #[test]
    fn multi_router() {
        use crate::multi_router::MultiRoute;

        mod isolated {
            #![allow(unused)]

            use crate::multi_router::{
                impl_multi_router_intercept_trait, multi_router, multi_router_intercept_trait,
            };

            multi_router_intercept_trait!(pub LifeIntercept for i32 | bool);
            multi_router_intercept_trait!(pub Empty for ());

            multi_router!(
                #[derive()]
                pub PlayerMultiRouter {
                    i as LifeIntercept where i32 => () | bool => (),
                    e as Empty where () => ()
                } else {
                    String => String
                }
            );

            multi_router!(pub ShieldedMultiRouter {
                i as LifeIntercept where i32 | bool
            });

            // implements `LifeIntercept` for `ShieldedMultiRouter`
            impl_multi_router_intercept_trait!(ShieldedMultiRouter as LifeIntercept for i32 | bool);
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Player {
            Alive { health: i32 },
            Dead,
        }
        impl Player {
            pub fn alive(&self) -> Option<&i32> {
                if let Self::Alive { health } = self {
                    Some(health)
                } else {
                    None
                }
            }

            pub fn dead(&self) -> Option<()> {
                if let Self::Dead = self {
                    Some(())
                } else {
                    None
                }
            }
        }
        impl Receive<i32> for Player {
            type Output = ();

            fn send(&mut self, event: i32) -> ReceiverResult<i32, Self::Output> {
                if let Self::Alive { health } = self {
                    *health += event;
                };
                ReceiverResult::Continue(())
            }
        }
        impl Receive<bool> for Player {
            type Output = ();

            fn send(&mut self, event: bool) -> ReceiverResult<bool, Self::Output> {
                if !event {
                    *self = Self::Dead;
                }
                ReceiverResult::Continue(())
            }
        }
        impl Receive<String> for Player {
            type Output = String;

            fn send(&mut self, event: String) -> ReceiverResult<String, Self::Output> {
                ReceiverResult::Continue(format!("recieved event: {}", event))
            }
        }
        impl Receive<()> for Player {
            type Output = ();

            fn send(&mut self, _: ()) -> ReceiverResult<(), Self::Output> {
                ReceiverResult::Continue(())
            }
        }
        impl std::fmt::Display for Player {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{}",
                    match self {
                        Player::Alive { health } => format!("Health: {}", health),
                        Player::Dead => "Dead".to_string(),
                    }
                )
            }
        }

        #[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
        pub struct Shielded {
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
        impl Receive<bool> for Shielded {
            type Output = bool;

            fn send(&mut self, event: bool) -> ReceiverResult<bool, Self::Output> {
                if self.shielded {
                    ReceiverResult::Stop
                } else {
                    ReceiverResult::Continue(event)
                }
            }
        }

        let mut multi_router = isolated::PlayerMultiRouter::new(Player::Alive { health: 100 });

        assert!(multi_router.send(-10).is_continue());
        assert_eq!(*multi_router.get_receiver().alive().unwrap(), 90);

        let shielded_linker = RcLinker::new(Shielded { shielded: false });
        multi_router.intercept(Box::new(isolated::ShieldedMultiRouter::new(
            shielded_linker.linked(),
        )) as Box<dyn isolated::LifeIntercept>);

        assert!(multi_router.send(-10).is_continue());
        assert_eq!(*multi_router.get_receiver().alive().unwrap(), 80);

        shielded_linker.borrow_mut().as_mut().unwrap().shielded = true;

        assert!(multi_router.send(false).is_stop());
        assert_eq!(*multi_router.get_receiver().alive().unwrap(), 80);

        let _: Box<dyn isolated::LifeIntercept> = multi_router.take_intercept().unwrap();

        assert!(multi_router.send(false).is_continue());
        assert_eq!(multi_router.get_receiver().dead(), Some(()));

        assert_eq!(
            multi_router
                .send("no intercept".to_string())
                .unwrap_continue(),
            "recieved event: no intercept"
        );

        multi_router = isolated::PlayerMultiRouter::new(Player::Alive { health: 100 });

        multi_router.intercept(Box::new(isolated::ShieldedMultiRouter::new(
            shielded_linker.linked(),
        )) as Box<dyn isolated::LifeIntercept>);

        multi_router.intercept(Box::new(isolated::ShieldedMultiRouter::new(
            shielded_linker.linked(),
        )) as Box<dyn isolated::LifeIntercept>);

        assert!(multi_router.send(false).is_stop());
        assert_eq!(*multi_router.get_receiver().alive().unwrap(), 100);

        println!("{}", multi_router);
        println!("{:?}", multi_router);

        (&mut multi_router as &mut dyn MultiRoute<dyn isolated::LifeIntercept>)
            .delete_top_intercept();

        assert!(multi_router.send(false).is_stop());
        assert_eq!(*multi_router.get_receiver().alive().unwrap(), 100);

        multi_router.intercept(Box::new(isolated::ShieldedMultiRouter::new(
            shielded_linker.linked(),
        )) as Box<dyn isolated::LifeIntercept>);

        drop(shielded_linker);

        assert!(multi_router.send(false).is_continue());
        assert_eq!(multi_router.get_receiver().dead(), Some(()));
    }

    #[test]
    fn multi_exposed() {
        mod isolated {
            crate::multi_exposed::multi_exposed_trait!(pub View1 for i32 | bool);
            crate::multi_exposed::multi_exposed_trait!(pub View2 for ());

            crate::multi_exposed::multi_exposed!(
                #[derive()]
                pub MultiExposed {
                    view1 as View1 for i32 => () | bool => (),
                    view2 as View2 for () => ()
                } else {
                    String => String
                }
            );
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Player {
            Alive { health: i32 },
            Dead,
        }
        impl Player {
            pub fn alive(&self) -> Option<&i32> {
                if let Self::Alive { health } = self {
                    Some(health)
                } else {
                    None
                }
            }

            pub fn dead(&self) -> Option<()> {
                if let Self::Dead = self {
                    Some(())
                } else {
                    None
                }
            }
        }
        impl Receive<i32> for Player {
            type Output = ();

            fn send(&mut self, event: i32) -> ReceiverResult<i32, Self::Output> {
                if let Self::Alive { health } = self {
                    *health += event;
                };
                ReceiverResult::Continue(())
            }
        }
        impl Receive<bool> for Player {
            type Output = ();

            fn send(&mut self, event: bool) -> ReceiverResult<bool, Self::Output> {
                if !event {
                    *self = Self::Dead;
                }
                ReceiverResult::Continue(())
            }
        }
        impl Receive<String> for Player {
            type Output = String;

            fn send(&mut self, event: String) -> ReceiverResult<String, Self::Output> {
                ReceiverResult::Continue(format!("recieved event: {}", event))
            }
        }
        impl Receive<()> for Player {
            type Output = ();

            fn send(&mut self, _: ()) -> ReceiverResult<(), Self::Output> {
                ReceiverResult::Continue(())
            }
        }
        impl std::fmt::Display for Player {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{}",
                    match self {
                        Player::Alive { health } => format!("Health: {}", health),
                        Player::Dead => "Dead".to_string(),
                    }
                )
            }
        }

        #[derive(Default, Clone, Debug, PartialEq, Eq)]
        pub struct Viewer1 {
            pub string_vec: Vec<String>,
        }
        impl View<i32> for Viewer1 {
            fn view(&mut self, event: &i32) -> Option<crate::view::DeleteView> {
                self.string_vec.push(format!("saw i32: {}", event));
                None
            }
        }
        impl View<bool> for Viewer1 {
            fn view(&mut self, event: &bool) -> Option<crate::view::DeleteView> {
                self.string_vec.push(format!("saw bool: {}", event));
                None
            }
        }

        #[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
        pub struct Viewer2 {
            pub count: u8,
        }
        impl View<()> for Viewer2 {
            fn view(&mut self, _: &()) -> Option<crate::view::DeleteView> {
                self.count += 1;
                None
            }
        }

        let mut multi_router = isolated::MultiExposed::new(Player::Alive { health: 100 });

        assert!(multi_router.send(-10).is_continue());
        assert_eq!(*multi_router.get_receiver().alive().unwrap(), 90);

        let viewer1_linker = RcLinker::new(Viewer1::default());
        let _ = multi_router
            .add_viewer(Box::new(viewer1_linker.linked()) as Box<dyn isolated::View1>)
            .unwrap();

        assert!(multi_router.send(-10).is_continue());
        assert_eq!(*multi_router.get_receiver().alive().unwrap(), 80);

        println!("{:?}", viewer1_linker);
        drop(viewer1_linker);

        let viewer2_linker = RcLinker::new(Viewer2 { count: 0 });
        let _ = multi_router
            .add_viewer(Box::new(viewer2_linker.linked()) as Box<dyn isolated::View2>)
            .unwrap();

        assert!(multi_router.send(false).is_continue());
        assert_eq!(multi_router.get_receiver().dead(), Some(()));

        assert_eq!(
            multi_router
                .send("no intercept".to_string())
                .unwrap_continue(),
            "recieved event: no intercept"
        );
    }
}
