use crate::size::Size;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Sizing {
    Intrinsic,
    Fill,
    Constrained(u16),
}

#[derive(Clone, Copy)]
pub enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy)]
pub struct Layout {
    pub sizing: Sizing,
    pub direction: Direction,
}

impl Layout {
    pub const fn new(sizing: Sizing, direction: Direction) -> Self {
        Layout { sizing, direction }
    }
}

pub trait IntrinsicSize {
    fn intrinsic_size(&self) -> Size;
}
