use embedded_graphics::draw_target::DrawTarget;

pub mod group;

pub use embedded_gui_macros::any_component;

use crate::{
    interactive,
    // layout::IntrinsicSize,
    primitive::Primitive,
    signal::Reactive,
    view::{Children, Factory, View},
};

/// Trait for reusable UI components.
///
/// A component receives children (widgets produced by its parent) and
/// produces a [`View`] that arranges them. Components are reactive –
/// they implement [`Reactive`] – so the framework knows when to re-render.
pub trait Component<
    'a,
    T: DrawTarget,
    Event,
    GlobalMsg,
    GlobalFocusKey: interactive::Key,
    AnyComponent: Component<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
    AnyPrimitive: Primitive<T>,
>: Reactive
{
    /// Builds a [`View`] from the component's current state, children, and the factory.
    fn view(
        self,
        v: &'a Factory<Event, GlobalMsg, GlobalFocusKey>,
        children: Children<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
    ) -> View<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>;
}
