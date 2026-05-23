use embedded_graphics::prelude::{Dimensions, DrawTarget};

use crate::{
    interactive::{self, FocusState},
    position::Position,
    view::{self, Factory, View},
};

pub trait App {
    type Target: DrawTarget;
    type Msg;
    type Event;
    type FocusKey: interactive::Key;

    fn new() -> Self;

    fn initial_focus_key() -> Self::FocusKey;

    fn background_color() -> <Self::Target as DrawTarget>::Color;
    fn update(&mut self, msg: Self::Msg) -> Option<(Self::FocusKey, FocusState)>;
    fn view<'a>(
        &'a self,
        v: &'a view::Factory<Self::FocusKey, Self::Event, Self::Msg>,
    ) -> View<'a, Self::Target, Self::FocusKey, Self::Event, Self::Msg>;
}

pub struct InternalState<FocusKey: interactive::Key, Event, Msg> {
    current_focus_key: FocusKey,
    previous_focus_key: Option<FocusKey>,
    factory: Factory<FocusKey, Event, Msg>,
}

impl<FocusKey: interactive::Key, Event, Msg> InternalState<FocusKey, Event, Msg> {
    pub fn new(focus_key: FocusKey) -> Self {
        Self {
            current_focus_key: focus_key,
            previous_focus_key: None,
            factory: Factory::new(focus_key),
        }
    }
}

pub fn dispatch<A: App>(
    app: &mut A,
    internal_state: &mut InternalState<A::FocusKey, A::Event, A::Msg>,
    events: impl IntoIterator<Item = A::Event>,
) {
    for event in events {
        let updated_focus = internal_state
            .factory
            .dispatch(event)
            .and_then(|msg| app.update(msg));

        updated_focus.map(|(key, state)| {
            internal_state.previous_focus_key = Some(internal_state.current_focus_key);
            internal_state.current_focus_key = key;

            internal_state.factory.set_focus(key, state);
        });
    }
}

pub fn render<A: App>(
    app: &A,
    internal_state: &mut InternalState<A::FocusKey, A::Event, A::Msg>,
    display: &mut A::Target,
) -> Result<(), <A::Target as DrawTarget>::Error> {
    let view = app.view(&internal_state.factory);

    let output = view.render(
        &internal_state.factory,
        Position::zero(),
        display.bounding_box().size.into(),
        internal_state.current_focus_key,
        internal_state.previous_focus_key,
        display,
        A::background_color(),
    );

    internal_state.factory.bump.reset();

    output
}
