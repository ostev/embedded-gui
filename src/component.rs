use crate::{arena::Arena, signal::Reactive};

pub trait Component<'a>: Reactive {
    fn view(&self, arena: &mut Arena);
}
