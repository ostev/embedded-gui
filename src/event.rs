use core::cell::RefCell;

use alloc::boxed::Box;
use esp_println::println;
use hashbrown::HashMap;

use crate::interactive;

pub(crate) struct Handler<Event, Msg> {
    handler: Box<dyn Fn(Event) -> Option<Msg>>,
}

impl<Event, Msg> Handler<Event, Msg> {
    pub const fn new(handler: Box<dyn Fn(Event) -> Option<Msg>>) -> Self {
        Self { handler }
    }
}

pub(crate) struct HandlerRegistry<FocusKey: interactive::Key, Event, Msg> {
    handles: RefCell<HashMap<FocusKey, Handler<Event, Msg>>>,
}

impl<FocusKey: interactive::Key, Event, Msg> HandlerRegistry<FocusKey, Event, Msg> {
    pub fn new() -> Self {
        Self {
            handles: RefCell::new(HashMap::new()),
        }
    }

    pub(crate) fn register(&self, key: FocusKey, handler: Handler<Event, Msg>) {
        self.handles.borrow_mut().insert(key, handler);
    }

    pub(crate) fn dispatch(&self, focus_key: &FocusKey, event: Event) -> Option<Msg> {
        let handles = self.handles.borrow();
        let handler = handles.get(focus_key);
        let msg = handler
            .map(|Handler { handler }| (handler)(event))
            .flatten();
        msg
    }
}
