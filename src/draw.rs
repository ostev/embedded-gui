use alloc::vec;
use alloc::vec::Vec;
use bumpalo::Bump;
use embedded_graphics::prelude::{DrawTargetExt, PixelColor};
use embedded_graphics::primitives::Rectangle;

use crate::position::Position;
use crate::size::Size;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pixel<Color: PixelColor> {
    pub position: Position,
    pub color: Color,
}

impl<Color: PixelColor> Pixel<Color> {
    pub fn new(position: Position, color: Color) -> Self {
        Self { position, color }
    }

    pub fn with_position(self, position: Position) -> Self {
        Self {
            position,
            color: self.color,
        }
    }

    pub fn with_color(self, color: Color) -> Self {
        Self {
            position: self.position,
            color: color,
        }
    }
}

pub(crate) struct TargetStore<Color: PixelColor> {
    buffer: Vec<Color>,
    width: usize,
    height: usize,
}

impl<Color: PixelColor> TargetStore<Color> {
    pub fn new(background: Color, width: usize, height: usize) -> Self {
        Self {
            buffer: vec![background; width * height],
            width,
            height,
        }
    }
}

pub struct LocalTarget<'a, Color: PixelColor> {
    buffer: bumpalo::collections::Vec<'a, Color>,
    position: Position,
    width: usize,
    size: Size,
    bottom_right: Position,
}

impl<'a, Color: PixelColor> LocalTarget<'a, Color> {
    /// Creates a new draw target. If the provided size is zero in either dimension,
    /// it returns `None` instead.
    pub(crate) fn try_new(
        bump: &'a Bump,
        background: Color,
        position: Position,
        size: Size,
    ) -> Option<Self> {
        size.bottom_right().map(|bottom_right| {
            let width: usize = size.width.into();
            let height: usize = size.height.into();
            Self {
                buffer: bumpalo::vec![in bump; background; width * height],
                position,
                width,
                size,
                bottom_right,
            }
        })
    }

    pub fn draw(&mut self, pixel: Pixel<Color>) {
        let clamped = self.bottom_right.min(pixel.position) + self.position;

        let row_offset = clamped.x as usize * self.width;
        let column_offset = clamped.y as usize;
        self.buffer[row_offset + column_offset] = pixel.color;
    }

    pub fn draw_iter(&mut self, pixels: impl Iterator<Item = Pixel<Color>>) {
        pixels.for_each(|pixel| self.draw(pixel));
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn blit<T: embedded_graphics::draw_target::DrawTarget<Color = Color>>(
        self,
        target: &mut T,
    ) -> Result<(), T::Error> {
        target.fill_contiguous(
            &Rectangle::new(self.position.into(), self.size.into()),
            self.buffer,
        )
    }
}
