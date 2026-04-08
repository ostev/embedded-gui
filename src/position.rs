#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
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

impl core::ops::Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl core::ops::Sub for Position {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl core::ops::Mul for Position {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl core::ops::Div for Position {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl core::cmp::Ord for Position {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.x.cmp(&other.x).then(self.y.cmp(&other.y))
    }
}

impl core::cmp::PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl From<embedded_graphics::prelude::Point> for Position {
    fn from(value: embedded_graphics::prelude::Point) -> Self {
        Position::new(value.x as u16, value.y as u16)
    }
}

impl Into<embedded_graphics::prelude::Point> for Position {
    fn into(self) -> embedded_graphics::prelude::Point {
        embedded_graphics::prelude::Point::new(self.x.into(), self.y.into())
    }
}
