use embedded_graphics::draw_target::DrawTarget;

use crate::{draw::LocalTarget, layout::IntrinsicSize, signal::Reactive};

pub mod spacer;
pub mod text;

pub trait Primitive<T: DrawTarget>: Reactive + IntrinsicSize {
    fn draw(&self, target: &mut LocalTarget<T>) -> Result<(), T::Error>;
}
