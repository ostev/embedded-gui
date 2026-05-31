use bumpalo::boxed::Box;
use embedded_graphics::draw_target::DrawTarget;

use crate::{
    component::Component,
    interactive,
    layout::{Direction, IntrinsicSize},
    primitive::Primitive,
    signal::{Reactive, SignalRef},
    size::Size,
    view::{self, Children},
};

#[derive(Reactive)]
pub struct Group<'model> {
    direction: SignalRef<'model, Direction>,
    size: SignalRef<'model, Size>,
}

impl<'model> Group<'model> {
    pub fn zero(direction: SignalRef<'model, Direction>) -> Self {
        Self {
            size: SignalRef::owned(Size::zero()),
            direction,
        }
    }
}

impl<'model> IntrinsicSize for Group<'model> {
    fn intrinsic_size(&self) -> crate::size::Size {
        *self.size
    }
}

impl<
    'a,
    'model,
    T: DrawTarget,
    Event,
    Msg,
    FocusKey: interactive::Key,
    AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    AnyPrimitive: Primitive<T>,
> Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive> for Group<'model>
{
    fn view(
        &self,
        v: &'a view::Factory<Event, Msg, FocusKey>,
        children: Children<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    ) -> crate::view::View<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive> {
        v.view_ref(*self.direction, children)
    }
}
