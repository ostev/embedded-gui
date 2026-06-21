use core::fmt::Debug;

use alloc::{borrow::Cow, boxed::Box, string::String};
use bumpalo::Bump;
use embedded_graphics::{draw_target::DrawTarget, mono_font::MonoTextStyle, prelude::PixelColor};

use crate::{
    component::Component,
    interactive,
    layout::{Direction, IntrinsicSize, Sizing},
    primitive::{Primitive, spacer::Spacer, text::Text},
    signal::{Reactive, Signal, SignalRef},
    size::Size,
    view::{self, Children},
};

#[derive(Reactive)]
pub struct Button<'model, Color: PixelColor, S: AsRef<str> + Clone> {
    pub text: SignalRef<'model, S>,
    pub font_style: Signal<MonoTextStyle<'static, Color>>,
    pub size: Signal<Size>,
}

// #[derive(Reactive)]
// enum A<'model> {
//     B(SignalRef<'model, u32>, SignalRef<'model, u16>),
//     C { a: SignalRef<'model, u32> },
// }

impl<'model, Color: PixelColor, S: AsRef<str> + Clone> IntrinsicSize for Button<'model, Color, S> {
    fn intrinsic_size(&self) -> Size {
        *self.size
    }
}

impl<
    'a,
    Color: PixelColor + 'a + Debug,
    T: DrawTarget<Color = Color>,
    Event,
    Msg,
    FocusKey: interactive::Key,
    AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    AnyPrimitive: Primitive<T>,
    S: AsRef<str> + Clone + 'a,
> Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive> for Button<'a, Color, S>
where
    Box<Self, &'a Bump>: Into<AnyComponent>,
    Box<Spacer, &'a Bump>: Into<AnyPrimitive>,
    Box<Text<'a, T::Color, S>, &'a Bump>: Into<AnyPrimitive>,
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
