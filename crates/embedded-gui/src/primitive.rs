use embedded_graphics::draw_target::DrawTarget;

use crate::{
    draw::LocalTarget,
    signal::{self, Reactive, Signal},
};

pub mod spacer;
pub mod text;

pub use embedded_gui_macros::any_primitive;

/// Trait for widgets that are drawn directly to the screen.
///
/// These are the leaves of the view tree, with `draw` rendering
/// themselves onto a [`LocalTarget`].
pub trait Primitive<T: DrawTarget>: Reactive {
    /// Draws this primitive onto the provided [`LocalTarget`].
    fn draw(self, p: &PrimitiveContext, target: &mut LocalTarget<T>) -> Result<(), T::Error>;
}

#[non_exhaustive]
pub struct PrimitiveContext();

impl PrimitiveContext {
    pub(crate) const fn new() -> PrimitiveContext {
        PrimitiveContext()
    }
}
