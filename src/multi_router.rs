#![allow(unused)]

macro_rules! list_helper {
    ($f:ident, $last:ident, $value:expr) => {
        write!($f, "{}: {}",stringify!($last), $value)
    };
    ($f:ident, $first:ident, $value:expr, $($rest:ident, $rest_value:expr),*) => {
        write!($f, "{}: {}, ",stringify!($first), $value)?;
        $crate::event_horizon::multi_router::list_helper!($f, $($rest, $rest_value),*)
    }
}

pub trait MultiRoute<I: ?Sized> {
    fn take_intercept(&mut self) -> Option<Box<I>>;
    fn delete_top_intercept(&mut self) -> Option<Box<I>>;
    fn intercept(&mut self, intercept: Box<I>);
}

#[allow(unused)]
macro_rules! multi_router_intercept_trait {
    ($vis:vis $I:ident for $($E:ty)|*) => {
        $vis trait $I: $($crate::event_horizon::receive::Receive<$E, Output = $E>+)* {
            fn take_intercept(&mut self) -> Option<Box<dyn $I>>;
            fn intercept(&mut self, intercept: Box<dyn $I>);
        }
    };
}

#[allow(unused)]
macro_rules! impl_multi_router_intercept_trait {
    ($Name:ident as $I:ident for $($E:ty)|*) => {
        impl<R: $($crate::event_horizon::receive::Receive<$E, Output = $E>+)*> $I for $Name<R> {
            fn take_intercept(&mut self) -> Option<Box<dyn $I>> {
                (self as &mut dyn crate::event_horizon::multi_router::MultiRoute<dyn $I>).take_intercept()
            }
            fn intercept(&mut self, intercept: Box<dyn $I>) {
                (self as &mut dyn crate::event_horizon::multi_router::MultiRoute<dyn $I>).intercept(intercept)
            }
        }
    };
}

#[allow(unused)]
macro_rules! multi_router {
    (#[derive($($attr:ident),*)] $vis:vis $Name:ident { $($intercept:ident as $I:ident where $($E:ty => $Output:ty)|*),* } else { $($P:ty => $POutput:ty),* }) => {
        #[derive($($attr),*)]
        $vis struct $Name<R> {
            $($intercept: Option<Box<dyn $I>>,)*
            receiver: R,
        }

        impl<R> $Name<R> {
            pub fn new(receiver: R) -> Self {
                Self { receiver, $($intercept: None),* }
            }

            #[allow(unused)]
            pub fn get_receiver(&self) -> &R {
                &self.receiver
            }

            #[allow(unused)]
            pub fn get_receiver_mut(&mut self) -> &mut R {
                &mut self.receiver
            }

            $(pub fn $intercept (&self) -> Option<&dyn $I> {
                self.$intercept.as_ref().map(Box::as_ref)
            })*
        }

        $($(impl<R: $crate::event_horizon::receive::Receive<$E, Output = $Output>> $crate::event_horizon::receive::Receive<$E> for $Name<R> {
            type Output = $Output;

            fn send(&mut self, event: $E) -> $crate::event_horizon::receive::ReceiverResult<$E, Self::Output> {
                let event = if let Some(ref mut intercept) = self.$intercept {
                    match intercept.send(event) {
                        $crate::event_horizon::receive::ReceiverResult::Continue(event) => event,
                        $crate::event_horizon::receive::ReceiverResult::Stop => return $crate::event_horizon::receive::ReceiverResult::Stop,
                        $crate::event_horizon::receive::ReceiverResult::Delete(event) => {
                            (self as &mut dyn $crate::event_horizon::multi_router::MultiRoute<dyn $I>).delete_top_intercept().unwrap();
                            event
                        }
                    }
                } else {
                    event
                };

                self.receiver.send(event)
            }
        })*)*

        $(impl<R: $crate::event_horizon::receive::Receive<$P, Output = $POutput>> $crate::event_horizon::receive::Receive<$P> for $Name<R> {
            type Output = $POutput;

            fn send(&mut self, event: $P) -> $crate::event_horizon::receive::ReceiverResult<$P, Self::Output> {
                self.receiver.send(event)
            }
        })*

        $(impl<R> $crate::event_horizon::multi_router::MultiRoute<dyn $I> for $Name<R> {
            fn take_intercept(&mut self) -> Option<Box<dyn $I>> {
                self.$intercept.take()
            }

            fn delete_top_intercept(&mut self) -> Option<Box<dyn $I>> {
                let mut old_intercept = self.take_intercept();
                if let Some(ref mut intercept) = old_intercept {
                    self.$intercept = (intercept as &mut Box<dyn $I>).take_intercept();
                }
                old_intercept
            }

            fn intercept(&mut self, intercept: Box<dyn $I>) {
                match self.$intercept {
                    Some(ref mut child) => child.intercept(intercept),
                    None => self.$intercept = Some(intercept),
                }
            }
        })*

        impl<R> Default for $Name<R> where R: Default {
            fn default() -> Self {
                Self::new(R::default())
            }
        }

        impl<R> std::fmt::Debug for $Name<R> where R: std::fmt::Debug {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "[")?;
                $crate::event_horizon::multi_router::list_helper!(f, $($intercept, match self.$intercept.as_ref() {
                    Some(_) => "active",
                    None => "none",
                }),*);
                write!(f, "], ")?;
                write!(f, "{:?}", self.receiver)
            }
        }

        impl<R> std::fmt::Display for $Name<R> where R: std::fmt::Display {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.receiver.fmt(f)
            }
        }
    };
    (#[derive($($attr:ident),*)] $vis:vis $Name:ident { $($intercept:ident as $I:ident where $($E:ty => $Output:ty)|*),* }) => {
        $crate::event_horizon::multi_router::multi_router!(#[derive($($attr),*)] $vis $Name { $($intercept as $I where $($E => $Output)|*),* } else {});
    };
    (#[derive($($attr:ident),*)] $vis:vis $Name:ident { $($intercept:ident as $I:ident where $($E:ty)|*),* } else { $($P:ty),* }) => {
        $crate::event_horizon::multi_router::multi_router!(#[derive($($attr),*)] $vis $Name { $($intercept as $I where $($E => $E)|*),* } else { $($P => $P),* });
    };
    (#[derive($($attr:ident),*)] $vis:vis $Name:ident { $($intercept:ident as $I:ident where $($E:ty)|*),* }) => {
        $crate::event_horizon::multi_router::multi_router!(#[derive($($attr),*)] $vis $Name { $($intercept as $I where $($E)|*),* } else {});
    };
    ($vis:vis $Name:ident { $($intercept:ident as $I:ident where $($E:ty => $Output:ty)|*),* } else { $($P:ty => $POutput:ty),* }) => {
        $crate::event_horizon::multi_router::multi_router!(#[derive()] $vis $Name { $($intercept as $I where $($E => $Output)|*),* } else { $($P => $POutput),* });
    };
    ($vis:vis $Name:ident { $($intercept:ident as $I:ident where $($E:ty => $Output:ty)|*),* }) => {
        $crate::event_horizon::multi_router::multi_router!(#[derive()] $vis $Name { $($intercept as $I where $($E => $Output)|*),* } else {});
    };
    ($vis:vis $Name:ident { $($intercept:ident as $I:ident where $($E:ty)|*),* } else { $($P:ty),* }) => {
        $crate::event_horizon::multi_router::multi_router!(#[derive()] $vis $Name { $($intercept as $I where $($E => $E)|*),* } else { $($P => $P),* });
    };
    ($vis:vis $Name:ident { $($intercept:ident as $I:ident where $($E:ty)|*),* }) => {
        $crate::event_horizon::multi_router::multi_router!(#[derive()] $vis $Name { $($intercept as $I where $($E)|*),* } else {});
    };
}

pub(crate) use impl_multi_router_intercept_trait;
pub(crate) use list_helper;
pub(crate) use multi_router;
pub(crate) use multi_router_intercept_trait;
