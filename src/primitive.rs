use embedded_graphics::draw_target::DrawTarget;

use crate::{draw::LocalTarget, layout::IntrinsicSize, signal::Reactive};

pub mod owned_text;
pub mod spacer;
pub mod text;

pub use embedded_gui_macros::any_primitive;

pub trait Primitive<T: DrawTarget>: Reactive + IntrinsicSize {
    fn draw(&self, target: &mut LocalTarget<T>) -> Result<(), T::Error>;
}
