use embedded_graphics::draw_target::DrawTarget;

use crate::{
    primitive::{Primitive, PrimitiveContext},
    signal::{Reactive, Signal},
    size::Size,
};

/// A spacer primitive that takes up space but draws nothing.
///
/// Useful for creating gaps or centering widgets inside a group.
#[derive(Reactive)]
pub struct Spacer<'a> {
    /// The space this spacer occupies.
    pub size: Signal<'a, Size>,
}

impl<'a> Spacer<'a> {
    /// Creates a zero-sized spacer. You'd normally apply fill sizing to this.
    pub fn zero() -> Self {
        Self {
            size: Signal::owned_constant(Size::zero()),
        }
    }
}

// impl IntrinsicSize for Spacer {
//     fn intrinsic_size(&self) -> Size {
//         *self.size
//     }
// }

impl<'a, T: DrawTarget> Primitive<T> for Spacer<'a> {
    fn draw(
        self,
        _p: &PrimitiveContext,
        _target: &mut crate::draw::LocalTarget<T>,
    ) -> Result<(), T::Error> {
        Ok(())
    }
}
