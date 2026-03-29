use bumpalo::Bump;

use crate::{primitive::Primitive, signal::Reactive, view::View};

pub trait Component<'a, P: Primitive, C>: Reactive {
    fn view(&self, bump: &Bump, children: View<'a, P, Self>)
    where
        Self: Sized;
}
