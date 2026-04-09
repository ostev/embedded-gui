use embedded_graphics::prelude::{DrawTarget, PixelColor};

use crate::{
    Reactive,
    draw::LocalTarget,
    primitive::Primitive,
    signal::{Reactive, SignalRef},
};

pub trait Background<Color: PixelColor> {
    fn draw<'a>(&self, target: &mut LocalTarget<'a, Color>);
}

#[derive(Reactive)]
pub struct Fill<'model, Color: PixelColor> {
    pub color: SignalRef<'model, Color>,
}

impl<'model, Color: PixelColor> Background<Color> for Fill<'model, Color> {
    fn draw<'a>(&self, target: &mut LocalTarget<'a, Color>) {
        let Ok(_) = target.clear(*self.color);
    }
}
#[derive(Reactive)]
pub struct Layer<'model, 'a, const N: usize, Color: PixelColor> {
    pub backgrounds: SignalRef<'model, [&'a dyn Primitive<Color>; N]>,
}

impl<'model, 'a, const N: usize, Color: PixelColor> Layer<'model, 'a, N, Color> {
    pub fn new(backgrounds: SignalRef<'model, [&'a dyn Primitive<Color>; N]>) -> Self {
        Self { backgrounds }
    }
}

impl<'model, 'a, const N: usize, Color: PixelColor> Background<Color>
    for Layer<'model, 'a, N, Color>
{
    fn draw<'b>(&self, target: &mut LocalTarget<'b, Color>) {
        for background in self.backgrounds.iter() {
            background.draw(target);
        }
    }
}
