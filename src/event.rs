use alloc::boxed::Box;
use hashbrown::HashMap;

use crate::interactive;

pub struct Handler<Event, Msg> {
    pub(crate) handler: Box<dyn Fn(Event) -> Msg>,
}

impl<Event, Msg> Handler<Event, Msg> {
    pub const fn new(handler: Box<dyn Fn(Event) -> Msg>) -> Self {
        Self { handler }
    }
}

pub struct HandlerRegistry<FocusKey: interactive::Key, Event, Msg> {
    handles: HashMap<FocusKey, Handler<Event, Msg>>,
}

impl<FocusKey: interactive::Key, Event, Msg> HandlerRegistry<FocusKey, Event, Msg> {
    pub fn new() -> Self {
        Self {
            handles: HashMap::new(),
        }
    }

    pub fn register(&mut self, key: FocusKey, handler: Handler<Event, Msg>) {
        self.handles.insert(key, handler);
    }
}
