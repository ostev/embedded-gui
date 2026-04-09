use embedded_graphics::{mono_font::MonoTextStyle, prelude::PixelColor};

use crate::{
    Reactive,
    component::Component,
    layout::{Direction, IntrinsicSize, Sizing},
    primitive::{spacer::Spacer, text::Text},
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

impl<'a, 'model, Color: PixelColor> Component<'a, Color> for Group<'model> {
    fn view(
        &self,
        v: &'a crate::view::Factory,
        children: &'a mut [crate::view::Widget<'a, Color>],
    ) -> crate::view::View<'a, Color> {
        v.view_ref(*self.direction, children)
    }
}
