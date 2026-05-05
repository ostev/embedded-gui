use core::convert::Infallible;
use std::marker::PhantomData;

use alloc::vec;
use alloc::vec::Vec;
use bumpalo::Bump;
use embedded_graphics::Pixel;
use embedded_graphics::prelude::{Dimensions, DrawTarget, OriginDimensions, PixelColor, Point};
use embedded_graphics::primitives::Rectangle;

use crate::position::Position;
use crate::size::Size;

pub(crate) struct Framebuffer<Color: PixelColor> {
    buffer: Vec<Color>,
    width: usize,
    height: usize,
}

impl<Color: PixelColor> Framebuffer<Color> {
    pub fn new(background: Color, width: usize, height: usize) -> Self {
        Self {
            buffer: vec![background; width * height],
            width,
            height,
        }
    }

    const fn index(&self, x: usize, y: usize) -> usize {
        x + y * self.width
    }

    pub fn blit<T: DrawTarget<Color = Color>>(&self, target: &mut T) -> Result<(), T::Error> {
        self.blit_with(target, |color| *color)
    }

    pub fn blit_with<T: DrawTarget<Color = TargetColor>, TargetColor: PixelColor>(
        &self,
        target: &mut T,
        f: impl FnMut(&Color) -> TargetColor,
    ) -> Result<(), T::Error> {
        target.fill_contiguous(
            &Rectangle::new(
                Point::zero(),
                embedded_graphics::geometry::Size::new(self.width as u32, self.height as u32),
            ),
            self.buffer.iter().map(f),
        )
    }
}

pub struct LocalTarget<'a, Color: PixelColor> {
    target: &'a mut Framebuffer<Color>,
    position: Position,
    size: Size,
    bottom_right: Position,
}

impl<'a, Color: PixelColor> LocalTarget<'a, Color> {
    /// Creates a new draw target. If the provided size is zero in either dimension,
    /// it returns `None` instead.
    pub(crate) fn try_new(
        target: &'a mut Framebuffer<Color>,
        position: Position,
        size: Size,
    ) -> Option<Self> {
        size.bottom_right().map(|bottom_right| Self {
            target,
            position,
            size,
            bottom_right,
        })
    }

    pub fn draw(&mut self, pixel: Pixel<Color>) {
        let Pixel(position, color) = pixel;

        if (position.x < 0)
            | (position.x > self.bottom_right.x as i32)
            | (position.y < 0)
            | (position.y > self.bottom_right.y as i32)
        {
            return;
        }

        let absolute_x = position.x as usize + self.position.x as usize;
        let absolute_y = position.y as usize + self.position.y as usize;

        let index = self.target.index(absolute_x, absolute_y);
        self.target.buffer[index] = color;
    }

    pub fn size(&self) -> Size {
        self.size
    }

    fn local_bounds(&self) -> Rectangle {
        Rectangle::new(Point::zero(), self.size.into())
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
        let max_x = self.size.width as i32;
        let max_y = self.size.height as i32;
        let mut colors = colors.into_iter();

        for row in 0..area.size.height {
            let local_y = area.top_left.y + row as i32;

            for col in 0..area.size.width {
                let color = match colors.next() {
                    Some(color) => color,
                    None => return Ok(()),
                };

                let local_x = area.top_left.x + col as i32;

                if (local_x < 0) | (local_x >= max_x) | (local_y < 0) | (local_y >= max_y) {
                    continue;
                }

                let absolute_x = local_x as usize + self.position.x as usize;
                let absolute_y = local_y as usize + self.position.y as usize;
                let index = self.target.index(absolute_x, absolute_y);
                self.target.buffer[index] = color;
            }
        }

        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let clipped = area.intersection(&self.local_bounds());
        let row_width = clipped.size.width as usize;

        for row in 0..clipped.size.height {
            let local_y = clipped.top_left.y + row as i32;
            let absolute_y = local_y as usize + self.position.y as usize;
            let absolute_x = clipped.top_left.x as usize + self.position.x as usize;
            let start = self.target.index(absolute_x, absolute_y);
            let end = start + row_width;
            self.target.buffer[start..end].fill(color);
        }

        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let row_width = self.size.width as usize;

        for row in 0..self.size.height {
            let absolute_y = self.position.y as usize + row as usize;
            let absolute_x = self.position.x as usize;
            let start = self.target.index(absolute_x, absolute_y);
            let end = start + row_width;
            self.target.buffer[start..end].fill(color);
        }

        Ok(())
    }
}
