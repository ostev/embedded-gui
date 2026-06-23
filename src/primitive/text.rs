use core::fmt::Debug;

use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    mono_font::MonoTextStyle,
    prelude::{PixelColor, Point},
    text::{Alignment, TextStyleBuilder},
};

use crate::{
    layout::IntrinsicSize,
    primitive::Primitive,
    signal::{Reactive, Signal, SignalRef},
    size::Size,
};

/// A text primitive that borrows its content.
///
/// Use this when the displayed text is a reference to app state and does
/// not need to be owned.
#[derive(Reactive)]
pub struct Text<'model, Color: PixelColor, S: AsRef<str>> {
    /// The text content (borrowed from app state).
    pub content: SignalRef<'model, S>,
    /// The font style used to render the text.
    pub font_style: Signal<MonoTextStyle<'static, Color>>,
}

impl<'model, 'a, Color: PixelColor, S: AsRef<str>> IntrinsicSize for Text<'model, Color, S> {
    fn intrinsic_size(&self) -> Size {
        let character_size: Size = self.font_style.font.character_size.into();
        let n_chars = self.content.as_ref().chars().count();
        let size = Size::new(character_size.width * n_chars as u16, character_size.height);
        size
    }
}

impl<'model, 'a, Color: PixelColor + Debug, T: DrawTarget<Color = Color>, S: AsRef<str>>
    Primitive<T> for Text<'model, Color, S>
{
    fn draw(&self, target: &mut crate::draw::LocalTarget<T>) -> Result<(), T::Error> {
        let text_style = TextStyleBuilder::new()
            .alignment(Alignment::Left)
            .baseline(embedded_graphics::text::Baseline::Top)
            .build();

        embedded_graphics::text::Text::with_text_style(
            self.content.as_ref(),
            Point::zero(),
            *self.font_style,
            text_style,
        )
        .draw(target)?;

        Ok(())
    }
}
