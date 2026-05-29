use core::any::Any;

use embedded_graphics::draw_target::DrawTarget;

use crate::{
    component::Component,
    interactive,
    layout::{Direction, IntrinsicSize},
    signal::{Reactive, SignalRef},
    size::Size,
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
    FocusKey: interactive::Key,
    Event,
    Msg,
    AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent>,
> Component<'a, T, Event, Msg, FocusKey, AnyComponent> for Group<'model>
{
    fn view<const N: usize>(
        self,
        v: &'a crate::view::Factory<FocusKey, Event, Msg>,
        children: [crate::view::Widget<'a, T, FocusKey, Event, Msg, AnyComponent>; N],
    ) -> crate::view::View<'a, T, FocusKey, Event, Msg> {
        v.view(*self.direction, children)
    }
}
