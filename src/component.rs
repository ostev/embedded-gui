use embedded_graphics::draw_target::DrawTarget;

pub mod button;
pub mod group;

use crate::{
    interactive,
    layout::IntrinsicSize,
    signal::Reactive,
    view::{self, View, Widget},
};

pub trait Component<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg>:
    Reactive + IntrinsicSize
{
    fn view(
        &self,
        v: &'a view::Factory,
        children: &'a mut [Widget<'a, T, FocusKey, Event, Msg>],
    ) -> View<'a, T, FocusKey, Event, Msg>;
}
