use alloc::string::String;
use embedded_graphics::{
    mono_font::MonoTextStyle,
    pixelcolor::Rgb888,
    prelude::{PixelColor, RgbColor},
};

use crate::{
    Reactive, background,
    component::Component,
    layout::{Direction, IntrinsicSize, Sizing},
    primitive::{spacer::Spacer, text::Text},
    signal::SignalRef,
    size::Size,
};

#[derive(Reactive)]
pub struct Button<'model, Color: PixelColor, S: AsRef<str>> {
    pub text: SignalRef<'model, S>,
    pub font_style: SignalRef<'model, MonoTextStyle<'static, Color>>,
    pub size: SignalRef<'model, Size>,
}

impl<'model, Color: PixelColor, S: AsRef<str>> IntrinsicSize for Button<'model, Color, S> {
    fn intrinsic_size(&self) -> crate::size::Size {
        *self.size
    }
}

impl<'a, 'model: 'a, Color: PixelColor, S: AsRef<str>> Component<'a, Color>
    for Button<'model, Color, S>
{
    fn view(
        &self,
        v: &'a crate::view::Factory,
        _children: &mut [crate::view::Widget<'a, Color>],
    ) -> crate::view::View<'a, Color> {
        v.view(
            Direction::Horizontal,
            [v.centered(
                Direction::Horizontal,
                v.primitive(
                    Sizing::Intrinsic,
                    Text {
                        content: self.text.clone(),
                        font_style: self.font_style.clone(),
                    },
                ),
            )],
        )
    }
}
