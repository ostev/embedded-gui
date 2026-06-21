use core::fmt::Debug;

use embedded_graphics::Pixel;
use embedded_graphics::prelude::{Dimensions, DrawTarget, OriginDimensions, PixelColor, Point};
use embedded_graphics::primitives::Rectangle;
use esp_println::println;

use crate::position::Position;
use crate::size::Size;

// pub(crate) struct Framebuffer<Color: PixelColor> {
//     buffer: Vec<Color>,
//     width: usize,
//     height: usize,
// }

// impl<Color: PixelColor> Framebuffer<Color> {
//     pub fn new(background: Color, width: usize, height: usize) -> Self {
//         Self {
//             buffer: vec![background; width * height],
//             width,
//             height,
//         }
//     }

//     const fn index(&self, x: usize, y: usize) -> usize {
//         x + y * self.width
//     }

//     pub fn blit<T: DrawTarget<Color = Color>>(&self, target: &mut T) -> Result<(), T::Error> {
//         self.blit_with(target, |color| *color)
//     }

//     pub fn blit_with<T: DrawTarget<Color = TargetColor>, TargetColor: PixelColor>(
//         &self,
//         target: &mut T,
//         f: impl FnMut(&Color) -> TargetColor,
//     ) -> Result<(), T::Error> {
//         target.fill_contiguous(
//             &Rectangle::new(
//                 Point::zero(),
//                 embedded_graphics::geometry::Size::new(self.width as u32, self.height as u32),
//             ),
//             self.buffer.iter().map(f),
//         )
//     }
// }

pub struct LocalTarget<'a, T: DrawTarget> {
    target: &'a mut T,
    position: Position,
    size: Size,
    bottom_right: Point,
}

impl<'a, T: DrawTarget> LocalTarget<'a, T> {
    /// Creates a new draw target. If the provided size is zero in either dimension,
    /// it returns `None` instead.
    pub fn try_new(target: &'a mut T, position: Position, size: Size) -> Option<Self> {
        size.bottom_right().map(|bottom_right| Self {
            target,
            position,
            size,
            bottom_right: bottom_right.into(),
        })
    }

    // pub fn draw(&mut self, pixel: Pixel<Color>) {

    // let index = self.target.index(absolute_x, absolute_y);
    // self.target.buffer[index] = color;
    // }

    pub fn size(&self) -> Size {
        self.size
    }

    fn local_bounds(&self) -> Rectangle {
        Rectangle::new(Point::zero(), self.size.into())
    }
}

impl<'a, T: DrawTarget> OriginDimensions for LocalTarget<'a, T> {
    fn size(&self) -> embedded_graphics::prelude::Size {
        self.size.into()
    }
}

const fn within_bounds(position: Point, bottom_right: Point) -> bool {
    #[cfg(feature = "clipping")]
    {
        (position.x >= 0)
            && (position.x <= bottom_right.x as i32)
            && (position.y >= 0)
            && (position.y <= bottom_right.y as i32)
    }
    #[cfg(not(feature = "clipping"))]
    {
        true
    }
}

impl<'a, Color: PixelColor + Debug, T: DrawTarget<Color = Color>> DrawTarget
    for LocalTarget<'a, T>
{
    type Color = Color;

    type Error = T::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        self.target.draw_iter(pixels.into_iter().map(|pixel| {
            let Pixel(position, color) = pixel;

            if within_bounds(position, self.bottom_right) {
                let absolute_x = position.x + self.position.x as i32;
                let absolute_y = position.y + self.position.y as i32;

                Pixel(Point::new(absolute_x, absolute_y), color)
            } else {
                panic!("Provided pixel position is out of bounds!")
            }
        }))?;

        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        // #[cfg(feature = "clipping")]
        // let clipped = area.intersection(&self.local_bounds());

        // #[cfg(not(feature = "clipping"))]
        // let clipped = area;

        // if area.

        // let absolute_top_left =
        //     clipped.top_left + Point::new(self.position.x as i32, self.position.y as i32);
        // let transformed_area = Rectangle::new(absolute_top_left, area.size);

        // self.target.fill_contiguous(&transformed_area, colors)
        todo!()
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        #[cfg(feature = "clipping")]
        let clipped = area.intersection(&self.local_bounds());

        #[cfg(not(feature = "clipping"))]
        let clipped = area;

        let absolute_top_left =
            clipped.top_left + Point::new(self.position.x as i32, self.position.y as i32);
        let transformed_area = Rectangle::new(absolute_top_left, clipped.size);
        // println!("Fill solid {:?} with color {:?}", transformed_area, color);

        self.target.fill_solid(&transformed_area, color)
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        // self.target.fill_solid(&self.bounding_box(), color)
        // println!("Local bounds: {:?}", self.local_bounds());
        self.fill_solid(&self.local_bounds(), color)
    }
}
