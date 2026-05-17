use alloc::boxed::Box;
use hashbrown::HashMap;

use crate::interactive;

// #[derive(Clone, Copy, Debug)]
// pub struct Handler<Event, Msg> {
//     handler: fn(Event) -> Msg,
// }

pub type Handler<Widget, Event, Msg> = fn(Widget, Event) -> Msg;

pub struct HandlerRegistry<FocusKey: interactive::Key, Event, Msg> {
    handles: HashMap<FocusKey, Handler<Event, Msg>>,
}

impl<FocusKey: interactive::Key, Event, Msg> HandlerRegistry<FocusKey, Event, Msg> {
    pub fn register(&mut self, key: FocusKey, handler: Handler<Event, Msg>) {
        self.handles.insert(key, handler);
    }
}
