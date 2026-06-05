use bumpalo::boxed::Box;
use embedded_graphics::draw_target::DrawTarget;

use crate::{
    component::Component,
    interactive,
    layout::{Direction, IntrinsicSize},
    primitive::Primitive,
    signal::{Reactive, Signal},
    size::Size,
    view::{self, Children},
};

#[derive(Reactive)]
pub struct Group {
    direction: Signal<Direction>,
    size: Signal<Size>,
}

impl Group {
    pub fn zero(direction: Signal<Direction>) -> Self {
        Self {
            size: Signal::constant(Size::zero()),
            direction,
        }
    }
}

impl IntrinsicSize for Group {
    fn intrinsic_size(&self) -> crate::size::Size {
        *self.size
    }
}

impl<
    'a,
    T: DrawTarget,
    Event,
    Msg,
    FocusKey: interactive::Key,
    AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    AnyPrimitive: Primitive<T>,
> Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive> for Group
{
    fn view(
        &self,
        v: &'a view::Factory<Event, Msg, FocusKey>,
        children: Children<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    ) -> crate::view::View<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive> {
        v.view_ref(*self.direction, children)
    }
}
