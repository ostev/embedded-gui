use crate::{
    arena::{self, Arena},
    node::Node,
    signal::Reactive,
};

pub trait Component<'a>: Reactive {
    fn view(&self, arena: &mut Arena<'a, Node<'a>>);
}
