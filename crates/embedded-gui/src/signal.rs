use core::{
    fmt::Debug,
    ops::{ControlFlow, Deref, FromResidual, Residual, Try},
};

use alloc::{borrow::Cow, boxed::Box, rc::Rc};
use bumpalo::Bump;
pub use embedded_gui_macros::Reactive;

use crate::app::State;

/// Trait for types that can report whether their value has changed.
///
/// Used by the reactive system to determine which parts of the UI need
/// to be re-rendered each frame.
pub trait Reactive {
    /// Returns `true` if the value has been modified since the last
    /// call to [`State::mark_resolved`].
    fn has_changed(&self) -> bool;
}

/// Helper macro for common trait impls
macro_rules! reactive_impl {
    ($t:ty) => {
        impl<'a, T> Reactive for $t
        where
            T: Reactive,
        {
            fn has_changed(&self) -> bool {
                T::has_changed(self)
            }
        }
    };
}

reactive_impl!(&'a T);

reactive_impl!(alloc::boxed::Box<T>);
reactive_impl!(alloc::boxed::Box<T, &'a Bump>);
reactive_impl!(alloc::rc::Rc<T>);
reactive_impl!(alloc::sync::Arc<T>);

reactive_impl!(bumpalo::boxed::Box<'a, T>);

impl<'a, T: Clone> Reactive for Cow<'a, T>
where
    T: Reactive,
{
    fn has_changed(&self) -> bool {
        T::has_changed(self)
    }
}

/// A mutable reactive value that tracks whether it has been modified.
///
/// Call [`Source::set`] or [`Source::update`] to mutate the value. You
/// can mark the change as being resolved by calling [`State::mark_resolved`],
/// but typically the only thing that should do this is the `embedded-gui`
/// runtime.
///
/// # Example
///
/// ```
/// let mut source = Source::new(42);
/// assert!(!source.has_changed());
/// source.set(100);
/// assert!(source.has_changed());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Source<T> {
    value: T,
    has_changed: bool,
}

impl<T> Source<T> {
    /// Creates a new [`Source`] with the given initial value. The change
    /// flag is initially `false`.
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            value,
            has_changed: false,
        }
    }

    // /// Creates a new [`Source`] with a custom initial change state. Make
    // /// sure that this `has_changed` state is correct, otherwise your application
    // /// will misbehave.
    // #[inline]
    // pub(crate) fn custom(value: T, has_changed: bool) -> Self {
    //     Self { value, has_changed }
    // }

    /// Replaces the stored value with `new_value` and marks the source as changed.
    #[inline]
    pub fn set(&mut self, new_value: T) {
        self.value = new_value;
        self.mark_changed();
    }

    /// Replaces the stored value with the result of `updater` and marks the source as changed.
    #[inline]
    pub fn set_with(&mut self, updater: impl FnOnce(&T) -> T) {
        self.value = updater(&self.value);
        self.mark_changed();
    }

    /// Mutates the stored value in place via `updater` and marks the source as changed.
    /// Returns the value returned by `updater`.
    #[inline]
    pub fn update<U>(&mut self, updater: impl FnOnce(&mut T) -> U) -> U {
        let output = updater(&mut self.value);
        self.mark_changed();

        output
    }

    #[inline(always)]
    fn mark_changed(&mut self) {
        self.has_changed = true;
    }

    pub fn signal<'a>(&'a self) -> Signal<'a, T> {
        Signal::new(SignalVariant::Borrowed {
            value: &self.value,
            has_changed: self.has_changed,
        })
    }

    /// Transforms the value by consuming `self` and applying `f`,
    /// preserving the change flag.
    pub fn map<U>(self, f: impl Fn(T) -> U) -> Source<U> {
        Source {
            value: f(self.value),
            has_changed: self.has_changed,
        }
    }

    /// Transforms the value by reference, preserving the change flag.
    pub fn map_ref<U>(&self, f: impl Fn(&T) -> U) -> Source<U> {
        Source {
            value: f(&self.value),
            has_changed: self.has_changed,
        }
    }
}

// impl<'a, T> Source<Option<T>> {
//     /// Returns a [`SignalRef`] to the inner value if this source contains
//     /// `Some`, or `None` otherwise.
//     pub fn option_signal_ref(&'a self) -> Option<Signal<'a, T>> {
//         self.value.as_ref().map(|value| {
//             Signal::new(SignalVariant::Borrowed { value, has_changed: () }Source {
//                 value,
//                 has_changed: self.has_changed,
//             }))
//         })
//     }
// }

impl<T> State for Source<T> {
    #[inline(always)]
    fn mark_resolved(&mut self) {
        self.has_changed = false;
    }
}

impl<T> Reactive for Source<T> {
    #[inline(always)]
    fn has_changed(&self) -> bool {
        self.has_changed
    }
}

impl<T> Deref for Source<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub struct Signal<'a, T> {
    variant: SignalVariant<'a, T>,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
enum SignalVariant<'a, T> {
    Owned(Source<T>),
    Borrowed { value: &'a T, has_changed: bool },
}

impl<'a, T> Signal<'a, T> {
    pub fn constant(value: &'static T) -> Self {
        Self::new(SignalVariant::Borrowed {
            value,
            has_changed: false,
        })
    }

    fn new(variant: SignalVariant<'a, T>) -> Self {
        Self { variant }
    }

    pub fn owned_constant(value: T) -> Self {
        Signal::new(SignalVariant::Owned(Source {
            value,
            has_changed: false,
        }))
    }

    pub fn map<'b, 'c, U>(&'b self, f: impl Fn(&T) -> U) -> Signal<'c, U> {
        Signal::new(SignalVariant::Owned(match &self.variant {
            SignalVariant::Owned(source) => Source {
                value: f(&source.value),
                has_changed: source.has_changed,
            },
            SignalVariant::Borrowed { value, has_changed } => Source {
                value: f(value),
                has_changed: *has_changed,
            },
        }))
    }
}

impl<'a, T> Signal<'a, T>
where
    T: Clone,
{
    pub fn map_clone<U>(self, f: impl Fn(T) -> U) -> Signal<'a, U> {
        Signal::new(SignalVariant::Owned(match self.variant {
            SignalVariant::Owned(source) => Source {
                value: f(source.value),
                has_changed: source.has_changed,
            },
            SignalVariant::Borrowed { value, has_changed } => Source {
                value: f(value.clone()),
                has_changed: has_changed,
            },
        }))
    }
}

impl<'a, T> Signal<'a, Option<T>>
where
    T: Clone,
{
    // /// If this `SignalRef` wraps `Some(value)`, returns a new `SignalRef`
    // /// to the inner value. Otherwise returns `None`.
    // pub fn to_option_ref(&self, bump: &'a Bump) -> Option<Signal<'a, T>> {
    //     let variant = match &self.variant {
    //         SignalVariant::Borrowed(source) => source.value.as_ref().map(|value| {
    //             SignalVariant::Borrowed(Source {
    //                 value,
    //                 has_changed: source.has_changed,
    //             })
    //         }),
    //         SignalVariant::Owned(source) => Option::as_ref(&source.value).map(|value| {
    //             SignalVariant::Owned(Source {
    //                 value: Rc::new(value.clone()),
    //                 has_changed: source.has_changed,
    //             })
    //         }),
    //     };

    //     variant.map(Signal::new)
    // }
}

impl<'a, T> Reactive for Signal<'a, T> {
    #[inline]
    fn has_changed(&self) -> bool {
        match &self.variant {
            SignalVariant::Owned(signal) => signal.has_changed(),
            SignalVariant::Borrowed { has_changed, .. } => *has_changed,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SignalResidual<'a, T>(Signal<'a, T>);

impl<'a, T> Residual<T> for SignalResidual<'a, T> {
    type TryType = Signal<'a, T>;
}

impl<'a, T> Try for Signal<'a, T> {
    type Output = T;

    type Residual = SignalResidual<'a, T>;

    fn from_output(output: T) -> Self {
        Signal::new(SignalVariant::Owned(Source {
            value: output,
            // Because we have successfully continued execution
            // without short-circuiting, the value has changed.
            has_changed: true,
        }))
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        if self.has_changed() {
            ControlFlow::Continue(self)
        } else {
            ControlFlow::Break(SignalResidual(self))
        }
    }
}

impl<'a, T> FromResidual for Signal<'a, T> {
    fn from_residual(residual: SignalResidual<'a, T>) -> Self {
        residual.0
    }
}

// impl<'a, T> Deref for Signal<'a, T> {
//     type Target = T;

//     #[inline]
//     fn deref(&self) -> &Self::Target {
//         match &self.variant {
//             SignalVariant::Owned(signal) => &signal.value,
//             SignalVariant::Borrowed(signal) => &signal.value,
//         }
//     }
// }
