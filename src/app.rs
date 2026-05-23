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

pub struct InternalState<FocusKey: interactive::Key> {
    current_focus_key: FocusKey,
    previous_focus_key: Option<FocusKey>,
}

pub async fn start<A: App, FutureEvent: Future<Output = impl Iterator<Item = A::Event>>>(
    mut app: A,
    display: &mut A::Target,
    mut receive_event: impl FnMut() -> FutureEvent,
    mut after_render: impl FnMut(),
) -> Result<(), <A::Target as DrawTarget>::Error> {
    let initial_focus_key = A::initial_focus_key();

    let mut factory = Factory::new(initial_focus_key);

    let mut internal_state = InternalState {
        current_focus_key: initial_focus_key,
        previous_focus_key: None,
    };

    loop {
        render(&app, &mut factory, &mut internal_state, display)?;
        after_render();

        let events = receive_event().await;

        for event in events {
            let updated_focus = factory.dispatch(event).and_then(|msg| app.update(msg));

            updated_focus.map(|(key, state)| {
                internal_state.previous_focus_key = Some(internal_state.current_focus_key);
                internal_state.current_focus_key = key;

                factory.set_focus(key, state);
            });
        }
    }
}

fn render<A: App>(
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
