pub trait Reactive {
    fn has_changed(&self) -> bool;
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
}

impl<T> Reactive for Signal<T> {
    fn has_changed(&self) -> bool {
        self.has_changed
    }
}
