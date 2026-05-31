use core::cell::RefCell;

use alloc::boxed::Box;
use hashbrown::HashMap;

use crate::interactive;

pub(crate) struct Handler<Event, Msg> {
    handler: Box<dyn Fn(Event) -> Msg>,
}

impl<Event, Msg> Handler<Event, Msg> {
    pub const fn new(handler: Box<dyn Fn(Event) -> Msg>) -> Self {
        Self { handler }
    }
}

pub struct HandlerRegistry<FocusKey: interactive::Key, Event, Msg> {
    handles: RefCell<HashMap<FocusKey, Handler<Event, Msg>>>,
}

impl<FocusKey: interactive::Key, Event, Msg> HandlerRegistry<FocusKey, Event, Msg> {
    pub fn new() -> Self {
        Self {
            handles: RefCell::new(HashMap::new()),
        }
    }

    pub fn register(&self, key: FocusKey, handler: Handler<Event, Msg>) {
        self.handles.borrow_mut().insert(key, handler);
    }

    pub fn dispatch(&self, focus_key: &FocusKey, event: Event) -> Option<Msg> {
        self.handles
            .borrow()
            .get(focus_key)
            .map(|Handler { handler }| (handler)(event))
    }
}
