use embedded_graphics::draw_target::DrawTarget;

pub mod button;
pub mod group;

use crate::{
    interactive,
    layout::IntrinsicSize,
    signal::Reactive,
    view::{Factory, View, Widget},
};

pub trait Component<'a, T: DrawTarget, Event, GlobalMsg, GlobalFocusKey: interactive::Key>:
    Reactive + IntrinsicSize
{
    fn view(
        &self,
        v: &'a Factory<GlobalFocusKey, Event, GlobalMsg>,
        children: &'a mut [Widget<'a, T, GlobalFocusKey, Event, GlobalMsg>],
    ) -> View<'a, T, GlobalFocusKey, Event, GlobalMsg>;
}
