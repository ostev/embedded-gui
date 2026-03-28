use crate::{arena, component::Component, primitive::Primitive};

pub enum Node<'a, P, C> {
    Component(&'a P),
    Primitive(&'a C),
}
