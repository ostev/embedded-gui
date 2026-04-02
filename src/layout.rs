use core::ops::Div;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}
impl Size {
    pub const fn new(width: u16, height: u16) -> Size {
        Size { width, height }
    }

    pub const fn zero() -> Size {
        Size {
            width: 0,
            height: 0,
        }
    }
}

impl core::ops::Div for Size {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            width: self.width / rhs.width,
            height: self.height / rhs.height,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Position {
    pub x: u16,
    pub y: u16,
}
impl Position {
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    pub const fn zero() -> Self {
        Self { x: 0, y: 0 }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Sizing {
    Intrinsic,
    Fill,
}

pub enum Direction {
    Horizontal,
    Vertical,
}

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
