use embedded_graphics::prelude::PixelColor;

use crate::{
    layout::IntrinsicSize,
    signal::Reactive,
    view::{self, View, Widget},
};

pub trait Component<FocusState, Color: PixelColor>: Reactive + IntrinsicSize {
    fn view<'a>(
        &self,
        v: &view::Factory,
        children: &[Widget<'a, FocusState, Color>],
    ) -> View<'a, FocusState, Color>;
}
