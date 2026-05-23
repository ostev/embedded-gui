use embedded_graphics::{draw_target::DrawTarget, mono_font::MonoTextStyle, prelude::PixelColor};

use crate::{
    Reactive,
    component::Component,
    interactive,
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
    FocusKey: interactive::Key,
    Event,
    Msg,
> Component<'a, T, Event, Msg, FocusKey> for Button<'model, Color, S>
{
    fn view(
        &self,
        v: &'a crate::view::GlobalFactory<FocusKey, Event, Msg>,
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
