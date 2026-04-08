use bumpalo::Bump;
use embedded_graphics::prelude::PixelColor;

use crate::{
    draw,
    layout::{IntrinsicSize, Layout},
    primitive::Primitive,
    signal::Reactive,
    view::{self, View, WidgetVariant},
};

pub trait Component<'a, FocusState, Color: PixelColor>: Reactive + IntrinsicSize {
    fn view(
        &self,
        v: &view::Factory,
        children: &[WidgetVariant<'a, FocusState, Color>],
    ) -> View<'a, FocusState, Color>;
}
