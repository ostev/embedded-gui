use crate::{
    draw,
    layout::{IntrinsicSize, Layout, Size},
    signal::Reactive,
};

pub trait Primitive: Reactive + IntrinsicSize {
    fn draw(&self, target: &mut draw::Target);
}
