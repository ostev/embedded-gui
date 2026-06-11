use core::{fmt::Debug, ops::Deref};

use alloc::{borrow::Cow, rc::Rc};
use bumpalo::{Bump, boxed::Box};
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
pub struct Source<T> {
    value: T,
    has_changed: bool,
}

impl<T> Source<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            value,
            has_changed: false,
        }
    }

    #[inline]
    pub fn set(&mut self, new_value: T) {
        self.value = new_value;
        self.mark_changed();
    }

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

    #[inline(always)]
    pub fn signal_ref<'a>(&'a self) -> SignalRef<'a, T> {
        SignalRef::new(SignalRefVariant::Borrowed(Source {
            value: &self.value,
            has_changed: self.has_changed,
        }))
    }

    pub fn map<U>(self, f: impl Fn(T) -> U) -> Source<U> {
        Source {
            value: f(self.value),
            has_changed: self.has_changed,
        }
    }

    pub fn map_ref<U>(&self, f: impl Fn(&T) -> U) -> Source<U> {
        Source {
            value: f(&self.value),
            has_changed: self.has_changed,
        }
    }
}

impl<T: Copy> Source<T> {
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

// impl<T: Debug> Debug for Signal<T> {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         f.debug_struct("Signal")
//             .field("value", &self.value)
//             .field("has_changed", &self.has_changed)
//             .finish()
//     }
// }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Signal<T: Copy> {
    source: Source<T>,
}

impl<T: Copy> Signal<T> {
    pub fn constant(value: T) -> Signal<T> {
        Signal {
            source: Source {
                value,
                has_changed: false,
            },
        }
    }

    pub fn map<U: Copy>(&self, f: impl Fn(T) -> U) -> Signal<U> {
        Signal {
            source: Source {
                value: f(self.source.value),
                has_changed: self.source.has_changed,
            },
        }
    }

    pub fn map_to_ref<'a, U>(&self, bump: &'a Bump, f: impl Fn(T) -> U) -> SignalRef<'a, U> {
        SignalRef::new(SignalRefVariant::Owned(Source {
            value: Rc::new_in(f(self.source.value), bump),
            has_changed: self.source.has_changed,
        }))
    }
}

impl<T: Copy> Reactive for Signal<T> {
    #[inline]
    fn has_changed(&self) -> bool {
        self.source.has_changed
    }
}

impl<T: Copy> Deref for Signal<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.source.value
    }
}

#[derive(PartialEq, Eq, Debug)]
enum SignalRefVariant<'a, T> {
    Owned(Source<Rc<T, &'a Bump>>),
    Borrowed(Source<&'a T>),
}

impl<'a, T> Clone for SignalRefVariant<'a, T> {
    fn clone(&self) -> Self {
        match self {
            SignalRefVariant::Owned(source) => SignalRefVariant::Owned(Source {
                value: source.value.clone(),
                has_changed: source.has_changed,
            }),
            SignalRefVariant::Borrowed(source) => SignalRefVariant::Borrowed(*source),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct SignalRef<'a, T> {
    variant: SignalRefVariant<'a, T>,
}

impl<'a, T> SignalRef<'a, T> {
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

    #[inline(always)]
    pub fn owned_constant(value: T, bump: &'a Bump) -> Self {
        SignalRef::new(SignalRefVariant::Owned(Source {
            value: Rc::new_in(value, bump),
            has_changed: false,
        }))
    }

    pub fn map_to_ref<U>(&self, bump: &'a Bump, f: impl Fn(&T) -> U) -> SignalRef<'a, U> {
        SignalRef::new(match &self.variant {
            SignalRefVariant::Owned(signal) => SignalRefVariant::Owned(Source {
                value: Rc::new_in(f(&signal.value), bump),
                has_changed: signal.has_changed,
            }),
            SignalRefVariant::Borrowed(signal) => SignalRefVariant::Owned(Source {
                value: Rc::new_in(f(&signal.value), bump),
                has_changed: signal.has_changed,
            }),
        })
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
