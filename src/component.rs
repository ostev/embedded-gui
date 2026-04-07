use bumpalo::Bump;

use crate::{
    draw,
    layout::{IntrinsicSize, Layout, Size},
    primitive::Primitive,
    signal::Reactive,
    view::{self, View, Widget},
};

pub trait Component<'a, FocusState>: Reactive + IntrinsicSize {
    fn view(
        &self,
        v: &view::Factory,
        children: &'a [Widget<'a, FocusState>],
    ) -> View<'a, FocusState>;
}
