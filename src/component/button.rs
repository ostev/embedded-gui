use alloc::{borrow::Cow, string::String};
use bumpalo::boxed::Box;
use embedded_graphics::{draw_target::DrawTarget, mono_font::MonoTextStyle, prelude::PixelColor};

use crate::{
    component::Component,
    interactive,
    layout::{Direction, IntrinsicSize, Sizing},
    primitive::{Primitive, spacer::Spacer, text::Text},
    signal::{Reactive, SignalRef},
    size::Size,
    view::{self, Children},
};

#[derive(Reactive)]
pub struct Button<'model, Color: PixelColor, S: AsRef<str>> {
    pub text: SignalRef<'model, S>,
    pub font_style: SignalRef<'model, MonoTextStyle<'static, Color>>,
    pub size: SignalRef<'model, Size>,
}

// #[derive(Reactive)]
// enum A<'model> {
//     B(SignalRef<'model, u32>, SignalRef<'model, u16>),
//     C { a: SignalRef<'model, u32> },
// }

impl<'model, Color: PixelColor, S: AsRef<str>> IntrinsicSize for Button<'model, Color, S> {
    fn intrinsic_size(&self) -> crate::size::Size {
        *self.size
    }
}

impl<
    'a,
    Color: PixelColor,
    T: DrawTarget<Color = Color>,
    Event,
    Msg,
    FocusKey: interactive::Key,
    AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    AnyPrimitive: Primitive<T>,
    S: AsRef<str> + 'a,
> Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive> for Button<'a, Color, S>
where
    Box<'a, Self>: Into<AnyComponent>,
    Box<'a, Spacer<'a>>: Into<AnyPrimitive>,
    Box<'a, Text<'a, T::Color, S>>: Into<AnyPrimitive>,
{
    fn view(
        &self,
        v: &'a view::Factory<Event, Msg, FocusKey>,
        _children: Children<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    ) -> crate::view::View<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive> {
        v.view(
            Direction::Horizontal,
            [
                v.spacer(),
                v.primitive(
                    Sizing::Intrinsic,
                    Text {
                        content: self.text.clone(),
                        font_style: self.font_style.clone(),
                    },
                ),
                v.spacer(),
            ],
        )
    }
}
