use core::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use alloc::{borrow::Cow, rc::Rc};
use bumpalo::{Bump, boxed::Box};
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

    /// Creates a new [`Source`] with a custom initial change state. Make
    /// sure that this `has_changed` state is correct, otherwise your application
    /// will misbehave.
    #[inline]
    pub(crate) fn custom(value: T, has_changed: bool) -> Self {
        Self { value, has_changed }
    }

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

    /// Returns a [`SignalRef`] that borrows the current value of this source.
    #[inline(always)]
    pub fn signal_ref<'a>(&'a self) -> SignalRef<'a, T> {
        SignalRef::new(SignalRefVariant::Borrowed(Source {
            value: &self.value,
            has_changed: self.has_changed,
        }))
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

impl<'a, T> Source<Option<T>> {
    /// Returns a [`SignalRef`] to the inner value if this source contains
    /// `Some`, or `None` otherwise.
    pub fn option_signal_ref(&'a self) -> Option<SignalRef<'a, T>> {
        self.value.as_ref().map(|value| {
            SignalRef::new(SignalRefVariant::Borrowed(Source {
                value,
                has_changed: self.has_changed,
            }))
        })
    }
}

impl<T: Copy> Source<T> {
    /// Returns a [`Signal`] (a copy-only reactive wrapper) for this source.
    #[inline(always)]
    pub fn signal(&self) -> Signal<T> {
        Signal { source: *self }
    }
}

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

/// An immutable copy of a reactive value. Created from a [`Source`] via
/// [`Source::signal`]. Useful for passing snapshot values into the view tree.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Signal<T> {
    source: Source<T>,
}

impl<T> Signal<T> {
    /// Creates a [`Signal`] that never reports as changed.
    pub fn constant(value: T) -> Signal<T> {
        Signal {
            source: Source {
                value,
                has_changed: false,
            },
        }
    }

    /// Transforms the value by reference, preserving the change flag.
    pub fn map<U>(&self, f: impl Fn(&T) -> U) -> Signal<U> {
        Signal {
            source: Source {
                value: f(&self.source.value),
                has_changed: self.source.has_changed,
            },
        }
    }

    /// Applies `f` to the value and stores the result in a bump-allocated
    /// [`SignalRef`], preserving the change flag.
    pub fn map_to_owned_ref<'a, U>(&self, bump: &'a Bump, f: impl Fn(&T) -> U) -> SignalRef<'a, U> {
        SignalRef::new(SignalRefVariant::Owned(Source {
            value: Rc::new(f(&self.source.value)),
            has_changed: self.source.has_changed,
        }))
    }
}

impl<T> Reactive for Signal<T> {
    #[inline]
    fn has_changed(&self) -> bool {
        self.source.has_changed
    }
}

impl<T> Deref for Signal<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.source.value
    }
}

#[derive(PartialEq, Eq, Debug)]
enum SignalRefVariant<'a, T> {
    // TODO: this isn't a great way to pass data around, and requires a bunch of
    // unnecessary heap allocation. Previously this was using a bump-allocated `Rc`,
    // but that broke in the most recent nightly. I'm planning on fully refactoring the
    // signal design, so I'll leave it as this for now.
    Owned(Source<Rc<T>>),
    Borrowed(Source<&'a T>),
}

impl<'a, T> Clone for SignalRefVariant<'a, T> {
    fn clone(&self) -> Self {
        match self {
            SignalRefVariant::Owned(source) => SignalRefVariant::Owned(Source {
                value: Rc::clone(&source.value),
                has_changed: source.has_changed,
            }),
            SignalRefVariant::Borrowed(source) => SignalRefVariant::Borrowed(*source),
        }
    }
}

/// A reactive reference to a value, either borrowed or owned (bump-allocated).
///
/// `SignalRef` is the primary way to pass data into the view tree without
/// copying. It can reference a field on the app state (borrowed) or own a
/// heap-allocated copy (owned).
#[derive(PartialEq, Eq, Debug)]
pub struct SignalRef<'a, T> {
    variant: SignalRefVariant<'a, T>,
}

impl<'a, T> SignalRef<'a, T> {
    /// Creates a [`SignalRef`] pointing to a static value that never changes.
    #[inline]
    pub fn constant(value: &'static T) -> Self {
        Self::new(SignalRefVariant::Borrowed(Source {
            value,
            has_changed: false,
        }))
    }

    #[inline(always)]
    fn new(variant: SignalRefVariant<'a, T>) -> Self {
        Self { variant }
    }

    /// Allocates the constant in the provided arena.
    #[inline(always)]
    pub fn owned_constant(value: T, bump: &'a Bump) -> Self {
        SignalRef::new(SignalRefVariant::Owned(Source {
            value: Rc::new(value),
            has_changed: false,
        }))
    }

    /// Transforms the value by-reference and stores the result in the bump,
    /// returning a new owned [`SignalRef`].
    pub fn map_ref<U>(&self, bump: &'a Bump, f: impl Fn(&T) -> U) -> SignalRef<'a, U> {
        SignalRef::new(match &self.variant {
            SignalRefVariant::Owned(signal) => SignalRefVariant::Owned(Source {
                value: Rc::new(f(&signal.value)),
                has_changed: signal.has_changed,
            }),
            SignalRefVariant::Borrowed(signal) => SignalRefVariant::Owned(Source {
                value: Rc::new(f(&signal.value)),
                has_changed: signal.has_changed,
            }),
        })
    }

    /// Transforms the value by-reference into a new [`Signal`] (an owned copy).
    pub fn map<U>(&self, f: impl Fn(&T) -> U) -> Signal<U> {
        Signal {
            source: match &self.variant {
                SignalRefVariant::Owned(signal) => Source {
                    value: f(&signal.value),
                    has_changed: signal.has_changed,
                },
                SignalRefVariant::Borrowed(signal) => Source {
                    value: f(&signal.value),
                    has_changed: signal.has_changed,
                },
            },
        }
    }

    /// Returns a [`Signal`] that borrows from this `SignalRef`.
    pub fn signal<'b>(&'b self) -> Signal<&'b T> {
        match &self.variant {
            SignalRefVariant::Owned(signal) => Signal {
                source: Source {
                    value: signal.value.as_ref(),
                    has_changed: signal.has_changed,
                },
            },
            SignalRefVariant::Borrowed(signal) => Signal {
                source: Source {
                    value: &signal.value,
                    has_changed: signal.has_changed,
                },
            },
        }
    }
}

impl<'a, T> SignalRef<'a, Option<T>>
where
    T: Clone,
{
    /// If this `SignalRef` wraps `Some(value)`, returns a new `SignalRef`
    /// to the inner value. Otherwise returns `None`.
    pub fn to_option_ref(&self, bump: &'a Bump) -> Option<SignalRef<'a, T>> {
        let variant = match &self.variant {
            SignalRefVariant::Borrowed(source) => source.value.as_ref().map(|value| {
                SignalRefVariant::Borrowed(Source {
                    value,
                    has_changed: source.has_changed,
                })
            }),
            SignalRefVariant::Owned(source) => Option::as_ref(&source.value).map(|value| {
                SignalRefVariant::Owned(Source {
                    value: Rc::new(value.clone()),
                    has_changed: source.has_changed,
                })
            }),
        };

        variant.map(SignalRef::new)
    }
}

impl<'a, T> Reactive for SignalRef<'a, T> {
    #[inline]
    fn has_changed(&self) -> bool {
        match &self.variant {
            SignalRefVariant::Owned(signal) => signal.has_changed(),
            SignalRefVariant::Borrowed(signal) => signal.has_changed(),
        }
    }
}

impl<'a, T> Deref for SignalRef<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match &self.variant {
            SignalRefVariant::Owned(signal) => &signal.value,
            SignalRefVariant::Borrowed(signal) => &signal.value,
        }
    }
}

impl<'a, T> Clone for SignalRef<'a, T> {
    fn clone(&self) -> Self {
        Self {
            variant: self.variant.clone(),
        }
    }
}
