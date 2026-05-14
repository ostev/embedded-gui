use bumpalo::Bump;
use embedded_graphics::prelude::{Dimensions, DrawTarget};

use crate::{
    interactive,
    position::Position,
    view::{self, View},
};

pub trait App {
    type Target: DrawTarget;
    type Msg;
    type FocusKey: Copy + Eq;

    fn init() -> Self;

    fn default_focus_state() -> interactive::FocusState;
    fn default_focus_key() -> Self::FocusKey;

    fn background_color() -> <Self::Target as DrawTarget>::Color;
    fn update(&mut self, msg: Self::Msg);
    fn view<'a>(&'a self, v: &'a view::Factory) -> View<'a, Self::Target, Self::FocusKey>;
}

pub fn init_and_render_once<A: App>(
    app: A,
    display: &mut A::Target,
) -> Result<(), <A::Target as DrawTarget>::Error> {
    display.clear(A::background_color())?;

    let factory = view::Factory { bump: Bump::new() };

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

pub fn render<A: App>(
    app: &A,
    factory: &mut view::Factory,
    display: &mut A::Target,
) -> Result<(), <A::Target as DrawTarget>::Error> {
    let view = app.view(factory);

    let output = view.render(
        factory,
        Position::zero(),
        display.bounding_box().size.into(),
        A::default_focus_key(),
        &A::default_focus_state(),
        false,
        display,
        A::background_color(),
    );

    factory.bump.reset();

    output
}
