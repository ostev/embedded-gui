use core::marker::PhantomData;

use crate::{component::Component, primitive::Primitive};

pub enum Widget<'a, P: Primitive, C: Component<'a, P>> {
    Component(C),
    Primitive(P),
    None,
    _PhantomLifetime(PhantomData<&'a ()>),
}

impl<'a, P: Primitive, C: Component<'a, P>> Widget<'a, P, C> {
    pub fn create_primitive_if_changed(primitive: P) -> Widget<'a, P, C> {
        if primitive.has_changed() {
            Widget::Primitive(primitive)
        } else {
            Widget::None
        }
    }
}

pub struct View<'a, P: Primitive, C: Component<'a, P>> {
    widgets: &'a [Widget<'a, P, C>],
}

pub trait Factory {}
