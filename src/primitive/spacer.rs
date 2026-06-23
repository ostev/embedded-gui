use embedded_graphics::draw_target::DrawTarget;

use crate::{
    layout::IntrinsicSize, primitive::Primitive, signal::Reactive, signal::Signal, size::Size,
};

/// A spacer primitive that takes up space but draws nothing.
///
/// Useful for creating gaps or centering widgets inside a group.
#[derive(Reactive)]
pub struct Spacer {
    /// The space this spacer occupies.
    pub size: Signal<Size>,
}

impl Spacer {
    /// Creates a zero-sized spacer. You'd normally apply fill sizing to this.
    pub fn zero() -> Self {
        Self {
            size: Signal::constant(Size::zero()),
        }
    }
}

impl IntrinsicSize for Spacer {
    fn intrinsic_size(&self) -> Size {
        *self.size
    }
}

impl<T: DrawTarget> Primitive<T> for Spacer {
    fn draw(&self, _target: &mut crate::draw::LocalTarget<T>) -> Result<(), T::Error> {
        Ok(())
    }
}
