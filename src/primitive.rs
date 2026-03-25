use crate::{draw, signal::Reactive};

pub trait Primitive: Reactive {
    fn draw(&self, target: impl draw::Target);
}
