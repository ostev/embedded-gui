use bumpalo::Bump;
use embedded_graphics::prelude::{Dimensions, DrawTarget};

use crate::{
    event::HandlerRegistry,
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
        v: &'a view::Factory<Self::FocusKey>,
    ) -> View<'a, Self::Target, Self::FocusKey, Self::Event, Self::Msg>;
}

pub fn init_and_render_once<A: App, FocusKey: interactive::Key>(
    app: A,
    display: &mut A::Target,
    focus_key: FocusKey,
) -> Result<(), <A::Target as DrawTarget>::Error> {
    display.clear(A::background_color())?;

    let factory = view::Factory::new();

    let view = app.view(&factory);

    display.clear(A::background_color())?;

    view.render(
        &factory,
        Position::zero(),
        display.bounding_box().size.into(),
        A::default_focus_key(),
        &A::default_focus_state(),
        false,
        display,
        A::background_color(),
    )?;

    Ok(())
}

pub struct InternalState<FocusKey: interactive::Key> {
    current_focus_key: FocusKey,
    previous_focus_key: FocusKey,
}

pub fn render<A: App>(
    app: &A,
    factory: &mut view::Factory<A::FocusKey>,
    internal_state: &mut InternalState<A::FocusKey, A::Event, A::Msg>,
    display: &mut A::Target,
) -> Result<(), <A::Target as DrawTarget>::Error> {
    let view = app.view(factory);

    let output = view.render(
        factory,
        &mut internal_state.handler_registry,
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
