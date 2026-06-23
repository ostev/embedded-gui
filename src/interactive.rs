use core::fmt::Debug;
use core::hash::Hash;

/// The focus state of an interactive widget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusState {
    /// The widget has keyboard focus.
    Focused,
    /// The widget does not have keyboard focus.
    Unfocused,
}

impl FocusState {
    /// Returns `true` if this state is [`Focused`](FocusState::Focused).
    #[inline]
    pub const fn is_focused(self) -> bool {
        match self {
            FocusState::Focused => true,
            FocusState::Unfocused => false,
        }
    }
}

/// Trait for types that can be used as a focus key.
///
/// Focus keys must be [`Copy`], [`Eq`], [`Hash`], and [`Debug`].
/// Any type satisfying those bounds automatically implements `Key`.
pub trait Key: Copy + Eq + Hash + Debug {}

impl<T: Copy + Eq + Hash + Debug> Key for T {}
