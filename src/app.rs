use alloc::{boxed::Box, vec::Vec};
use bumpalo::Bump;
use embedded_graphics::prelude::{DrawTarget, PixelColor};

use crate::{
    draw, interactive,
    position::Position,
    view::{self, View},
};

pub trait App {
    type Color: PixelColor;
    type Palette: Into<Self::Color> + PixelColor;
    type Msg;

    fn init() -> Self;

    fn default_focus_state() -> interactive::FocusState;
    fn background_color() -> Self::Palette;
    fn update(&mut self, msg: Self::Msg);
    fn view<'a>(&'a self, v: &'a view::Factory) -> View<'a, Self::Palette>;
}

pub fn start<'a, A: App, D: DrawTarget<Color = A::Color>>(
    app: A,
    display: &mut D,
) -> Result<(), D::Error> {
    display.clear(A::background_color().into())?;

    let factory = view::Factory { bump: Bump::new() };

    let view = app.view(&factory);
    let focus_key = interactive::Key::MAX;

    let mut framebuffer = draw::Framebuffer::new(
        A::background_color(),
        display.bounding_box().size.width as usize,
        display.bounding_box().size.height as usize,
    );

    let Ok(_) = view.render(
        &factory,
        Position::zero(),
        display.bounding_box().size.into(),
        focus_key,
        &A::default_focus_state(),
        false,
        &mut framebuffer,
        A::background_color(),
    );

    framebuffer.blit_with(display, |palette_color| (*palette_color).into())
}
