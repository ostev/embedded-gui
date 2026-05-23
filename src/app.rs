use embedded_graphics::prelude::{Dimensions, DrawTarget};

use crate::{
    interactive,
    position::Position,
    view::{self, View},
};

pub trait App {
    type Target: DrawTarget;
    type Msg;
    type Event;
    type FocusKey: interactive::Key;

    fn init() -> Self;

    fn default_focus_state() -> interactive::FocusState;
    fn default_focus_key() -> Self::FocusKey;

    fn background_color() -> <Self::Target as DrawTarget>::Color;
    fn update(&mut self, msg: Self::Msg);
    fn view<'a>(
        &'a self,
        v: &'a view::Factory<Self::FocusKey, Self::Event, Self::Msg>,
    ) -> View<'a, Self::Target, Self::FocusKey, Self::Event, Self::Msg>;
}

pub struct InternalState<FocusKey: interactive::Key> {
    current_focus_key: FocusKey,
    previous_focus_key: FocusKey,
}

pub fn render<A: App>(
    app: &A,
    factory: &mut view::Factory<A::FocusKey, A::Event, A::Msg>,
    internal_state: &mut InternalState<A::FocusKey>,
    display: &mut A::Target,
) -> Result<(), <A::Target as DrawTarget>::Error> {
    let view = app.view(factory);

    let output = view.render(
        factory,
        Position::zero(),
        display.bounding_box().size.into(),
        internal_state.current_focus_key,
        internal_state.previous_focus_key,
        display,
        A::background_color(),
    );

    factory.bump.reset();

    output
}
