use embedded_graphics::draw_target::DrawTarget;

pub mod button;
pub mod group;

use crate::{
    layout::IntrinsicSize,
    signal::Reactive,
    view::{self, View, Widget},
};

pub trait Component<'a, T: DrawTarget, FocusKey: Copy + Eq>: Reactive + IntrinsicSize {
    fn view(
        &self,
        v: &'a view::Factory,
        children: &'a mut [Widget<'a, T, FocusKey>],
    ) -> View<'a, T, FocusKey>;
}
