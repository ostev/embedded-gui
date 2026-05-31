use embedded_graphics::prelude::{Dimensions, DrawTarget};

use crate::{
    component::Component,
    interactive::{self, FocusState},
    position::Position,
    signal::Reactive,
    view::{self, Factory, View},
};

pub use embedded_gui_macros::State;

pub trait App: State + Reactive {
    type Target: DrawTarget;
    type Msg;
    type Event;
    type FocusKey: interactive::Key;
    type AnyComponent<'a>: Component<'a, Self::Target, Self::Event, Self::Msg, Self::FocusKey, Self::AnyComponent<'a>>
    where
        Self: 'a;

    fn new() -> Self;

    fn initial_focus_key() -> Self::FocusKey;

    fn background_color() -> <Self::Target as DrawTarget>::Color;
    fn update(&mut self, msg: Self::Msg) -> Option<(Self::FocusKey, FocusState)>;
    fn view<'a>(
        &'a self,
        v: &'a view::Factory<Self::Event, Self::Msg, Self::FocusKey>,
    ) -> View<'a, Self::Target, Self::Event, Self::Msg, Self::FocusKey, Self::AnyComponent<'a>>;
}

pub trait State {
    fn mark_resolved(&mut self);
}

pub struct InternalState<Event, Msg, FocusKey: interactive::Key> {
    current_focus_key: FocusKey,
    previous_focus_key: Option<FocusKey>,
    factory: Factory<Event, Msg, FocusKey>,
}

impl<Event, Msg, FocusKey: interactive::Key> InternalState<Event, Msg, FocusKey> {
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
    internal_state: &mut InternalState<A::Event, A::Msg, A::FocusKey>,
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
    app: &mut A,
    internal_state: &mut InternalState<A::Event, A::Msg, A::FocusKey>,
    display: &mut A::Target,
) -> Result<(), <A::Target as DrawTarget>::Error> {
    if app.has_changed() {
        let view = app.view(&internal_state.factory);

        // Safety: the view is rendered immediately after being built from this factory.
        let output = unsafe {
            view.render(
                &internal_state.factory,
                Position::zero(),
                display.bounding_box().size.into(),
                internal_state.current_focus_key,
                internal_state.previous_focus_key,
                display,
                A::background_color(),
            )
        };

        internal_state.factory.bump.reset();

        app.mark_resolved();

        output
    } else {
        Ok(())
    }
}
