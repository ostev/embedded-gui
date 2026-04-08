use embedded_graphics::prelude::PixelColor;

use crate::{draw, layout::IntrinsicSize, signal::Reactive, size::Size};

pub mod spacer;
pub mod text;

pub trait Primitive<Color: PixelColor>: Reactive + IntrinsicSize {
    fn draw(&self, target: &mut draw::LocalTarget<Color>);
}
