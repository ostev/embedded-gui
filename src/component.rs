use core::any::Any;

use embedded_graphics::draw_target::DrawTarget;

pub mod button;
pub mod group;

pub use embedded_gui_macros::any_component;

use crate::{
    interactive,
    layout::IntrinsicSize,
    signal::Reactive,
    view::{Factory, View, Widget},
};

pub trait Component<
    'a,
    T: DrawTarget,
    Event,
    GlobalMsg,
    GlobalFocusKey: interactive::Key,
    AnyComponent: Component<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent>,
>: Reactive + IntrinsicSize
{
    fn view<const N: usize>(
        self,
        v: &'a Factory<GlobalFocusKey, Event, GlobalMsg>,
        children: [Widget<'a, T, GlobalFocusKey, Event, GlobalMsg, AnyComponent>; N],
    ) -> View<'a, T, GlobalFocusKey, Event, GlobalMsg>;
}
