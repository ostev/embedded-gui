use alloc::{boxed::Box, vec::Vec};
use bumpalo::Bump;
use embedded_graphics::prelude::{DrawTarget, PixelColor};

use crate::{
    interactive,
    signal::Reactive,
    view::{self, View},
};

pub trait App {
    type FocusState;
    type Color: PixelColor;
    type Msg;

    fn init() -> Self;

    fn default_focus_state() -> Self::FocusState;
    fn background_color() -> Self::Color;
    fn update(&mut self, msg: Self::Msg);
    fn view<'a>(&'a self, v: &'a view::Factory) -> View<'a, Self::FocusState, Self::Color>;
}

pub fn start<'a, A: App, D: DrawTarget<Color = A::Color>>(
    app: A,
    display: &mut D,
) -> Result<(), D::Error> {
    display.clear(A::background_color())?;

    let factory = view::Factory { bump: Bump::new() };

    let view = app.view(&factory);
    let focus_key = interactive::Key::MAX;

    view.render(
        &factory,
        A::background_color(),
        display.bounding_box().size.into(),
        focus_key,
        &A::default_focus_state(),
        false,
        display,
    )?;

    Ok(())
}
