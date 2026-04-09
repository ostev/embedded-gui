use embedded_graphics::prelude::PixelColor;

pub mod button;
pub mod group;

use crate::{
    layout::IntrinsicSize,
    signal::Reactive,
    view::{self, View, Widget},
};

pub trait Component<'a, Color: PixelColor>: Reactive + IntrinsicSize {
    fn view(&self, v: &'a view::Factory, children: &'a mut [Widget<'a, Color>]) -> View<'a, Color>;
}
