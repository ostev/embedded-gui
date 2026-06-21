use core::fmt::Debug;

use bumpalo::boxed::Box;
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
pub struct Background<Color: PixelColor> {
    pub color: Signal<Color>,
}

impl<Color: PixelColor> IntrinsicSize for Background<Color> {
    fn intrinsic_size(&self) -> Size {
        Size::zero()
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
> Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive> for Background<Color>
where
    Box<'a, Self>: Into<AnyComponent>,
{
    fn view(
        &self,
        v: &'a view::Factory<Event, Msg, FocusKey>,
        children: Children<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    ) -> crate::view::View<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive> {
        v.view_ref(Direction::Horizontal, children)
            .with_background(*self.color)
    }
}
