use bumpalo::boxed::Box;
use embedded_graphics::draw_target::DrawTarget;

use crate::{
    component::Component,
    interactive,
    layout::{Direction, IntrinsicSize},
    primitive::Primitive,
    signal::{Reactive, Signal},
    size::Size,
    view::{self, Children},
};

/// A layout component that arranges children in a horizontal or vertical stack.
///
/// The `direction` signal controls whether children are laid out left-to-right
/// or top-to-bottom.
#[derive(Reactive)]
pub struct Group {
    /// The direction children are laid out in.
    pub direction: Signal<Direction>,
    /// The size of this group component.
    pub size: Signal<Size>,
}

impl Group {
    /// Creates a `Group` with zero size (intrinsic sizing will be used).
    pub fn zero(direction: Signal<Direction>) -> Self {
        Self {
            size: Signal::constant(Size::zero()),
            direction,
        }
    }
}

impl IntrinsicSize for Group {
    fn intrinsic_size(&self) -> crate::size::Size {
        *self.size
    }
}

impl<
    'a,
    T: DrawTarget,
    Event,
    Msg,
    FocusKey: interactive::Key,
    AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    AnyPrimitive: Primitive<T>,
> Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive> for Group
{
    fn view(
        &self,
        v: &'a view::Factory<Event, Msg, FocusKey>,
        children: Children<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    ) -> crate::view::View<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive> {
        v.view_ref(*self.direction, children)
    }
}
