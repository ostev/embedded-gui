use crate::size::Size;

/// Controls how a widget is sized in the view tree.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Sizing {
    /// Use the widget's intrinsic (natural) size.
    Intrinsic,
    /// Expand to fill the remaining available space.
    Fill,
    /// Use a fixed size in the layout direction (specified in pixels).
    Constrained(u16),
}

/// The direction widgets are laid out in.
#[derive(Clone, Copy)]
pub enum Direction {
    /// Lay out children left to right.
    Horizontal,
    /// Lay out children top to bottom.
    Vertical,
}

/// Combined sizing and direction information for a layout pass.
#[derive(Clone, Copy)]
pub struct Layout {
    /// The sizing strategy for this layout.
    pub sizing: Sizing,
    /// The layout direction.
    pub direction: Direction,
}

impl Layout {
    /// Creates a new `Layout` with the given sizing and direction.
    pub const fn new(sizing: Sizing, direction: Direction) -> Self {
        Layout { sizing, direction }
    }
}

/// Trait for types that have a natural intrinsic size.
pub trait IntrinsicSize {
    /// Returns the intrinsic size of this item.
    fn intrinsic_size(&self) -> Size;
}
