use embedded_graphics::draw_target::DrawTarget;

use crate::{draw::LocalTarget, layout::IntrinsicSize, signal::Reactive};

pub mod owned_text;
pub mod spacer;
pub mod text;

pub use embedded_gui_macros::any_primitive;

/// Trait for widgets that are drawn directly to the screen.
///
/// These are the leaves of the view tree, with `draw` rendering
/// themselves onto a [`LocalTarget`].
pub trait Primitive<T: DrawTarget>: Reactive + IntrinsicSize {
    /// Draws this primitive onto the provided [`LocalTarget`].
    fn draw(&self, target: &mut LocalTarget<T>) -> Result<(), T::Error>;
}
