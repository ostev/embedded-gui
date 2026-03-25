use crate::{arena, component::Component, primitive::Primitive};

pub enum Node<'a, P, C> {
    Component(arena::Box<'a>),
    Primitive(arena::Box<'a>),
}
