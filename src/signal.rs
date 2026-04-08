use core::{fmt::Debug, ops::Deref};

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
            has_changed: false,
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

    pub(crate) fn mark_resolved(&mut self) {
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

pub enum SignalRef<'a, T> {
    Owned(Signal<T>),
    Borrowed(&'a Signal<T>),
}

impl<'a, T> SignalRef<'a, T> {
    pub fn owned(value: T) -> Self {
        Self::Owned(Signal {
            value,
            has_changed: false,
        })
    }
}

impl<'a, T> Reactive for SignalRef<'a, T> {
    fn has_changed(&self) -> bool {
        match self {
            Self::Owned(signal) => signal.has_changed(),
            Self::Borrowed(signal) => signal.has_changed(),
        }
    }
}

impl<'a, T> Deref for SignalRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(signal) => &signal.value,
            Self::Borrowed(signal) => &signal.value,
        }
    }
}
