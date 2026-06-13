use embedded_graphics::draw_target::DrawTarget;

pub mod background;
pub mod button;
pub mod group;

pub use embedded_gui_macros::any_component;

use crate::{
    interactive,
    layout::IntrinsicSize,
    primitive::Primitive,
    signal::Reactive,
    view::{Children, Factory, View},
};

pub trait Component<
    'a,
    T: DrawTarget,
    Event,
    GlobalMsg,
    GlobalFocusKey: interactive::Key,
    AnyComponent: Component<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
    AnyPrimitive: Primitive<T>,
>: Reactive + IntrinsicSize
{
    fn view(
        &self,
        v: &'a Factory<Event, GlobalMsg, GlobalFocusKey>,
        children: Children<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
    ) -> View<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>;
}
