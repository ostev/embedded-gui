use embedded_graphics::draw_target::DrawTarget;

use crate::{
    Reactive,
    component::Component,
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

impl<'a, 'model, T: DrawTarget, FocusKey: Copy + Eq> Component<'a, T, FocusKey> for Group<'model> {
    fn view(
        &self,
        v: &'a crate::view::Factory,
        children: &'a mut [crate::view::Widget<'a, T, FocusKey>],
    ) -> crate::view::View<'a, T, FocusKey> {
        v.view_ref(*self.direction, children)
    }
}
