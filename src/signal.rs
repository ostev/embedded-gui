use core::{fmt::Debug, ops::Deref};

use alloc::borrow::Cow;
pub use embedded_gui_macros::Reactive;

use crate::app::State;

pub trait Reactive {
    fn has_changed(&self) -> bool;
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Signal<T> {
    value: T,
    has_changed: bool,
}

impl<T> Signal<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            value,
            has_changed: true,
        }
    }

    #[inline]
    pub fn set(&mut self, new_value: T) {
        self.value = new_value;
        self.mark_changed();
    }

    #[inline]
    pub fn update(&mut self, updater: impl Fn(&mut T) -> ()) {
        updater(&mut self.value);
        self.mark_changed();
    }

    #[inline(always)]
    fn mark_changed(&mut self) {
        self.has_changed = true;
    }

    #[inline(always)]
    pub fn to_ref<'a>(&'a self) -> SignalRef<'a, T> {
        SignalRef::Borrowed(self)
    }

    pub fn map<U>(self, f: impl Fn(T) -> U) -> Signal<U> {
        Signal {
            value: f(self.value),
            has_changed: self.has_changed,
        }
    }

    pub fn map_ref<U>(&self, f: impl Fn(&T) -> U) -> Signal<U> {
        Signal {
            value: f(&self.value),
            has_changed: self.has_changed,
        }
    }
}

impl<T> State for Signal<T> {
    #[inline(always)]
    fn mark_resolved(&mut self) {
        self.has_changed = false;
    }
}

impl<T> Reactive for Signal<T> {
    #[inline(always)]
    fn has_changed(&self) -> bool {
        self.has_changed
    }
}

// impl<T: Debug> Debug for Signal<T> {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         f.debug_struct("Signal")
//             .field("value", &self.value)
//             .field("has_changed", &self.has_changed)
//             .finish()
//     }
// }

#[derive(Copy, PartialEq, Eq, Debug)]
pub enum SignalRef<'model, T> {
    Owned(Signal<T>),
    Borrowed(&'model Signal<T>),
}

impl<'model, T> SignalRef<'model, T> {
    #[inline(always)]
    pub fn owned(value: T) -> Self {
        Self::Owned(Signal {
            value,
            has_changed: false,
        })
    }

    pub fn map<U>(&self, f: impl Fn(&T) -> U) -> SignalRef<'model, U> {
        match self {
            SignalRef::Owned(signal) => SignalRef::Owned(Signal {
                value: f(&signal.value),
                has_changed: signal.has_changed,
            }),
            SignalRef::Borrowed(signal) => SignalRef::Owned(Signal {
                value: f(&signal.value),
                has_changed: signal.has_changed,
            }),
        }
    }
}

impl<'model, T> Reactive for SignalRef<'model, T> {
    #[inline]
    fn has_changed(&self) -> bool {
        match self {
            Self::Owned(signal) => signal.has_changed(),
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

impl<'model, T: Clone> Clone for SignalRef<'model, T> {
    #[inline]
    fn clone(&self) -> Self {
        match self {
            Self::Owned(value) => Self::Owned(value.clone()),
            Self::Borrowed(signal) => Self::Borrowed(signal),
        }
    }
}
