#![allow(unused)]

use counted_map::ReassignableCountedMap;

pub trait MultiExpose<I: ?Sized> {
    fn get_viewers(&self) -> &ReassignableCountedMap<usize, Box<I>>;
    fn add_viewer(&mut self, other: Box<I>) -> Result<usize, counted_map::HashMapFull>;
    fn remove_viewer(&mut self, id: usize) -> Option<Box<I>>;
}

macro_rules! multi_exposed_trait {
    ($vis:vis $I:ident for $($E:ty)|*) => {
        $vis trait $I: $($crate::event_horizon::view::View<$E>+)* {}
        impl<I: $($crate::event_horizon::view::View<$E>+)*> $I for I {}
    };
}

macro_rules! multi_exposed {
    (#[derive($($attr:ident),*)] $vis:vis $Name:ident { $($viewers:ident as $I:ident for $($E:ty => $Output:ty)|*),* } else { $($P:ty => $POutput:ty),* }) => {
        #[derive($($attr),*)]
        $vis struct $Name<R> {
            $($viewers: $crate::event_horizon::counted_map::ReassignableCountedMap<usize, Box<dyn $I>>,)*
            receiver: R,
        }

        #[allow(unused)]
        impl<R> $Name<R> {
            pub fn new(receiver: R) -> Self {
                Self { receiver, $($viewers: $crate::event_horizon::counted_map::ReassignableCountedMap::new(),)* }
            }

            pub fn get_receiver(&self) -> &R {
                &self.receiver
            }

            pub fn get_receiver_mut(&mut self) -> &mut R {
                &mut self.receiver
            }

            $(pub fn $viewers(&self) -> &$crate::event_horizon::counted_map::ReassignableCountedMap<usize, Box<dyn $I>> {
                &self.$viewers
            })*
        }

        $($(impl<R: $crate::event_horizon::receive::Receive<$E, Output = $Output>> $crate::event_horizon::receive::Receive<$E> for $Name<R> {
            type Output = $Output;

            fn send(&mut self, event: $E) -> $crate::event_horizon::receive::ReceiverResult<$E, Self::Output> {
                let mut deleted = Vec::new();

                for (id, viewer) in self.$viewers.iter_mut() {
                    if viewer.view(&event).is_some() {
                        deleted.push(*id);
                    }
                }

                for id in deleted {
                    self.$viewers.remove(id);
                }

                self.receiver.send(event)
            }
        })*)*

        $(impl<R: $crate::event_horizon::receive::Receive<$P, Output = $POutput>> $crate::event_horizon::receive::Receive<$P> for $Name<R> {
            type Output = $POutput;

            fn send(&mut self, event: $P) -> $crate::event_horizon::receive::ReceiverResult<$P, Self::Output> {
                self.receiver.send(event)
            }
        })*

        $(impl<R> $crate::event_horizon::multi_exposed::MultiExpose<dyn $I> for $Name<R> {
            fn get_viewers(&self) -> &$crate::event_horizon::counted_map::ReassignableCountedMap<usize, Box<dyn $I>> {
                &self.$viewers
            }
            fn add_viewer(&mut self, other: Box<dyn $I>) -> Result<usize, $crate::event_horizon::counted_map::HashMapFull> {
                self.$viewers.push(other)
            }
            fn remove_viewer(&mut self, id: usize) -> Option<Box<dyn $I>> {
                self.$viewers.remove(id)
            }
        })*

        impl<R> Default for $Name<R> where R: Default {
            fn default() -> Self {
                Self::new(R::default())
            }
        }

        impl<R> std::fmt::Debug for $Name<R> where R: std::fmt::Debug {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{{[")?;
                $crate::event_horizon::multi_router::list_helper!(f, $($viewers, self.$viewers.len()),*);
                write!(f, "], ")?;
                write!(f, "{:?}}}", self.receiver)
            }
        }

        impl<R> std::fmt::Display for $Name<R> where R: std::fmt::Display {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.receiver.fmt(f)
            }
        }
    };
}

pub(crate) use multi_exposed;
pub(crate) use multi_exposed_trait;
