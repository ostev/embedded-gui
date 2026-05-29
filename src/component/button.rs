use alloc::rc::Rc;
use embedded_graphics::{draw_target::DrawTarget, mono_font::MonoTextStyle, prelude::PixelColor};

use crate::{
    component::Component,
    interactive,
    layout::{Direction, IntrinsicSize, Sizing},
    primitive::text::Text,
    signal::{Reactive, SignalRef},
    size::Size,
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
    'model: 'a,
    Color: PixelColor,
    S: AsRef<str>,
    T: DrawTarget<Color = Color>,
    FocusKey: interactive::Key,
    Event,
    Msg,
    AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent>,
> Component<'a, T, Event, Msg, FocusKey, AnyComponent> for Button<'model, Color, S>
where
    bumpalo::boxed::Box<'a, Self>: Into<AnyComponent>,
{
    fn view<const N: usize>(
        self,
        v: &'a crate::view::Factory<FocusKey, Event, Msg>,
        _children: [crate::view::Widget<'a, T, FocusKey, Event, Msg, AnyComponent>; N],
    ) -> crate::view::View<'a, T, FocusKey, Event, Msg> {
        v.view(
            Direction::Horizontal,
            [v.centered(
                Direction::Horizontal,
                v.primitive(
                    Sizing::Intrinsic,
                    Text {
                        content: self.text,
                        font_style: self.font_style.clone(),
                    },
                ),
            )],
        )
    }
}
