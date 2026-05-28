use core::{fmt::Debug, ops::Deref};

use alloc::rc::Rc;
use bumpalo::{Bump, boxed::Box};

pub use embedded_gui_macros::Reactive;

pub trait Reactive {
    fn has_changed(&self) -> bool;
}

impl<'a, T> Reactive for &'a T
where
    T: Reactive,
{
    fn has_changed(&self) -> bool {
        (*self).has_changed()
    }
}

pub struct Signal<T> {
    value: T,
    has_changed: bool,
}

impl<T> Signal<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            has_changed: true,
        }
    }

    pub fn set(&mut self, new_value: T) {
        self.value = new_value;
        self.mark_changed();
    }

    pub fn update(&mut self, updater: impl Fn(&mut T) -> ()) {
        updater(&mut self.value);
        self.mark_changed();
    }

    #[inline(always)]
    fn mark_changed(&mut self) {
        self.has_changed = true;
    }

    #[inline(always)]
    pub fn mark_resolved(&mut self) {
        self.has_changed = false;
    }

    pub fn to_ref<'a>(&'a self) -> SignalRef<'a, T> {
        SignalRef::Borrowed(self)
    }
}

impl<T> Reactive for Signal<T> {
    #[inline(always)]
    fn has_changed(&self) -> bool {
        self.has_changed
    }
}

impl<T: Debug> Debug for Signal<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Signal")
            .field("value", &self.value)
            .field("has_changed", &self.has_changed)
            .finish()
    }
}

pub enum SignalRef<'model, T> {
    Owned(Rc<Signal<T>>),
    Borrowed(&'model Signal<T>),
}

impl<'model, T> SignalRef<'model, T> {
    pub fn owned(value: T) -> Self {
        Self::Owned(Rc::new(Signal {
            value,
            has_changed: false,
        }))
    }
}

impl<'model, T> Reactive for SignalRef<'model, T> {
    fn has_changed(&self) -> bool {
        match self {
            Self::Owned(_) => false,
            Self::Borrowed(signal) => signal.has_changed(),
        }
    }
}

impl<'model, T> Deref for SignalRef<'model, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(signal) => &signal.value,
            Self::Borrowed(signal) => &signal.value,
        }
    }
}

impl<'model, T> Clone for SignalRef<'model, T> {
    fn clone(&self) -> Self {
        match self {
            Self::Owned(signal) => Self::Owned(signal.clone()),
            Self::Borrowed(signal) => Self::Borrowed(signal),
        }
    }
}
