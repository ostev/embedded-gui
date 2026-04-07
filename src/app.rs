use alloc::{boxed::Box, vec::Vec};

use crate::{
    signal::Reactive,
    view::{self, View},
};

pub trait App<'a, FocusState> {
    type Model: Reactive;
    type Msg;
    type Event;

    fn init() -> Self::Model;
    fn update(model: &mut Self::Model, msg: Self::Msg);
    fn view(v: &view::Factory<'a>) -> View<'a, FocusState>;
}
