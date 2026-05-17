use embedded_graphics::{draw_target::DrawTarget, mono_font::MonoTextStyle, prelude::PixelColor};

use crate::{
    Reactive,
    component::Component,
    layout::{Direction, IntrinsicSize, Sizing},
    primitive::text::Text,
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

impl<
    'a,
    'model: 'a,
    Color: PixelColor,
    S: AsRef<str>,
    T: DrawTarget<Color = Color>,
    FocusKey: Copy + Eq,
    Event,
    Msg,
> Component<'a, T, FocusKey, Event, Msg> for Button<'model, Color, S>
{
    fn view(
        &self,
        v: &'a crate::view::Factory,
        _children: &mut [crate::view::Widget<'a, T, FocusKey, Event, Msg>],
    ) -> crate::view::View<'a, T, FocusKey, Event, Msg> {
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
