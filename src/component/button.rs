use bumpalo::boxed::Box;
use embedded_graphics::{draw_target::DrawTarget, mono_font::MonoTextStyle, prelude::PixelColor};

use crate::{
    component::Component,
    interactive,
    layout::{Direction, IntrinsicSize, Sizing},
    primitive::text::Text,
    signal::{Reactive, SignalRef},
    size::Size,
    view::{self, Children},
};

#[derive(Reactive)]
pub struct Button<'model, Color: PixelColor> {
    pub text: SignalRef<'model, &'model str>,
    pub font_style: SignalRef<'model, MonoTextStyle<'static, Color>>,
    pub size: SignalRef<'model, Size>,
}

// #[derive(Reactive)]
// enum A<'model> {
//     B(SignalRef<'model, u32>, SignalRef<'model, u16>),
//     C { a: SignalRef<'model, u32> },
// }

impl<'model, Color: PixelColor> IntrinsicSize for Button<'model, Color> {
    fn intrinsic_size(&self) -> crate::size::Size {
        *self.size
    }
}

impl<
    'a,
    'model: 'a,
    Color: PixelColor,
    T: DrawTarget<Color = Color>,
    Event,
    Msg,
    FocusKey: interactive::Key,
    AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent>,
> Component<'a, T, Event, Msg, FocusKey, AnyComponent> for Button<'model, Color>
where
    Box<'a, Self>: Into<AnyComponent>,
{
    fn view(
        &self,
        v: &'a view::Factory<Event, Msg, FocusKey>,
        _children: Children<'a, T, Event, Msg, FocusKey, AnyComponent>,
    ) -> crate::view::View<'a, T, Event, Msg, FocusKey, AnyComponent> {
        v.view(
            Direction::Horizontal,
            [
                v.spacer(),
                v.primitive(
                    Sizing::Intrinsic,
                    Text {
                        content: self.text,
                        font_style: self.font_style,
                    },
                ),
                v.spacer(),
            ],
        )
    }
}
