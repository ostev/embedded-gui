use core::convert::Infallible;

use alloc::vec;
use alloc::vec::Vec;
use bumpalo::Bump;
use embedded_graphics::Pixel;
use embedded_graphics::prelude::{DrawTarget, DrawTargetExt, OriginDimensions, PixelColor, Point};
use embedded_graphics::primitives::Rectangle;

use crate::position::Position;
use crate::size::Size;

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
        let positive_x = pixel.0.x.max(0);
        let positive_y = pixel.0.y.max(0);
        let clamped = self
            .bottom_right
            .min(Position::new(positive_x as u16, positive_y as u16));

        self.buffer[(clamped.x as usize) + (clamped.y as usize) * self.width] = pixel.1;
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub(crate) fn blit<T: embedded_graphics::draw_target::DrawTarget<Color = Color>>(
        self,
        target: &mut T,
    ) -> Result<(), T::Error> {
        target.fill_contiguous(
            &Rectangle::new(self.position.into(), self.size.into()),
            self.buffer,
        )
    }
}

impl<'a, Color: PixelColor> OriginDimensions for LocalTarget<'a, Color> {
    fn size(&self) -> embedded_graphics::prelude::Size {
        self.size.into()
    }
}

impl<'a, Color: PixelColor> DrawTarget for LocalTarget<'a, Color> {
    type Color = Color;

    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        pixels.into_iter().for_each(|pixel| self.draw(pixel));

        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        todo!()
        // self.draw_iter(
        //     area.points()
        //         .zip(colors)
        //         .map(|(pos, color)| embedded_graphics::Pixel(pos, color)),
        // )
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        self.fill_contiguous(area, core::iter::repeat(color))
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.buffer.fill(color);

        Ok(())
    }
}
