use embedded_graphics::prelude::PixelColor;
use embedded_gui_macros::Reactive;

use crate::{
    layout::IntrinsicSize,
    primitive::Primitive,
    signal::{Signal, SignalRef},
    size::Size,
};

#[derive(Reactive)]
pub struct Spacer<'model> {
    pub size: SignalRef<'model, Size>,
}

impl<'model> IntrinsicSize for Spacer<'model> {
    fn intrinsic_size(&self) -> Size {
        *self.size
    }
}

impl<'model, Color: PixelColor> Primitive<Color> for Spacer<'model> {
    fn draw(&self, _target: &mut crate::draw::LocalTarget<Color>) {}
}
