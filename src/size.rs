use crate::position::Position;

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

    /// Returns the bottom right corner of the bounding box created
    /// by the `Size`. Note that this will wrap if either dimension
    /// of the size is zero.
    pub const fn bottom_right_unchecked(self) -> Position {
        Position {
            x: self.width - 1,
            y: self.height - 1,
        }
    }

    /// Returns the bottom right corner of the bounding box created
    /// by the `Size`, or `None` if either dimension of the size is zero.
    pub const fn bottom_right(self) -> Option<Position> {
        if (self.width == 0) | (self.height == 0) {
            None
        } else {
            Some(Position {
                x: self.width - 1,
                y: self.height - 1,
            })
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

impl From<embedded_graphics::prelude::Size> for Size {
    fn from(value: embedded_graphics::prelude::Size) -> Self {
        Size::new(value.width as u16, value.height as u16)
    }
}
impl Into<embedded_graphics::prelude::Size> for Size {
    fn into(self) -> embedded_graphics::prelude::Size {
        embedded_graphics::prelude::Size::new(self.width.into(), self.height.into())
    }
}
