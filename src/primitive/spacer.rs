use embedded_graphics::draw_target::DrawTarget;

use crate::{
    layout::IntrinsicSize, primitive::Primitive, signal::Reactive, signal::Signal, size::Size,
};

#[derive(Reactive)]
pub struct Spacer {
    pub size: Signal<Size>,
}

impl Spacer {
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
