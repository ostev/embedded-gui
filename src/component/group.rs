use embedded_graphics::draw_target::DrawTarget;

use crate::{
    Reactive,
    component::Component,
    interactive,
    layout::{Direction, IntrinsicSize},
    signal::SignalRef,
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

impl<'a, 'model, T: DrawTarget, FocusKey: interactive::Key, Event, Msg>
    Component<'a, T, Event, Msg, FocusKey> for Group<'model>
{
    fn view(
        &self,
        v: &'a crate::view::GlobalFactory<FocusKey, Event, Msg>,
        children: &'a mut [crate::view::Widget<'a, T, FocusKey, Event, Msg>],
    ) -> crate::view::View<'a, T, FocusKey, Event, Msg> {
        v.view_ref(*self.direction, children)
    }
}
