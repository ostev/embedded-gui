use core::{marker::PhantomData, num};

use alloc::vec;
use alloc::vec::Vec;
use bumpalo::{Bump, boxed::Box};
use embedded_gui_macros::Reactive;

use crate::{
    component::Component,
    draw,
    layout::{Align, Direction, Layout, Position, Size, Sizing},
    primitive::Primitive,
    signal::Reactive,
};

enum WidgetInner<'a> {
    Component(&'a dyn Component<'a>, &'a mut [Widget<'a>]),
    Primitive(&'a dyn Primitive),
}

impl<'a> WidgetInner<'a> {
    fn primitive(bump: &'a Bump, primitive: impl Primitive + 'a) -> Self {
        // if primitive.has_changed() {
        Self::Primitive(bump.alloc(primitive))
        // } else {
        // Self::None
        // }
    }

    fn component<const N: usize>(
        bump: &'a Bump,
        component: impl Component<'a> + 'a,
        children: [Widget<'a>; N],
    ) -> Self {
        // if component.has_changed() {
        Self::Component(bump.alloc(component), bump.alloc(children))
        // } else {
        // Self::None
        // }
    }

    fn component_ref(
        bump: &'a Bump,
        component: impl Component<'a> + 'a,
        children: &'a mut [Widget<'a>],
    ) -> Self {
        // if component.has_changed() {
        Self::Component(bump.alloc(component), children)
        // } else {
        // Self::None
        // }
    }
}

pub struct Widget<'a> {
    inner: WidgetInner<'a>,
    layout: Layout,
    size: Size,
    position: Position,
}

impl<'a> Widget<'a> {
    fn primitive(bump: &'a Bump, primitive: impl Primitive + 'a, layout: Layout) -> Self {
        Widget {
            inner: WidgetInner::primitive(bump, primitive),
            layout,
            size: Size::zero(),
            position: Position::zero(),
        }
    }

    fn component<const N: usize>(
        bump: &'a Bump,
        component: impl Component<'a> + 'a,
        layout: Layout,
        children: [Widget<'a>; N],
    ) -> Self {
        Widget {
            inner: WidgetInner::component(bump, component, children),
            layout,
            size: Size::zero(),
            position: Position::zero(),
        }
    }

    fn intrinsic_size(&self) -> Size {
        match self.inner {
            WidgetInner::Component(component, _) => component.intrinsic_size(),
            WidgetInner::Primitive(primitive) => primitive.intrinsic_size(),
        }
    }
}

pub struct Factory<'a> {
    pub bump: &'a Bump,
}
impl<'a> Factory<'a> {
    pub fn component<const N: usize>(
        &self,
        component: impl Component<'a> + 'a,
        layout: Layout,
        children: [Widget<'a>; N],
    ) -> Widget<'a> {
        Widget::component(self.bump, component, layout, children)
    }

    pub fn primitive(&self, primitive: impl Primitive + 'a, layout: Layout) -> Widget<'a> {
        Widget::primitive(self.bump, primitive, layout)
    }

    pub fn view<const N: usize>(&self, children: [Widget<'a>; N]) -> View<'a> {
        View {
            widgets: self.bump.alloc(children),
        }
    }

    pub fn view_ref(&self, children: &'a mut [Widget<'a>]) -> View<'a> {
        View { widgets: children }
    }
}

pub struct View<'a> {
    widgets: &'a mut [Widget<'a>],
}

impl<'a> View<'a> {
    fn compute_size(self, layout: Layout, available_space: Size) -> SizedView<'a> {
        let reduce_fill_space = match layout.direction {
            Direction::Horizontal => |fill_space: Size, size: Size| {
                Size::new(fill_space.width - size.width, fill_space.height)
            },
            Direction::Vertical => |fill_space: Size, size: Size| {
                Size::new(fill_space.width, fill_space.height - size.height)
            },
        };

        let (num_fill, fill_space) =
            self.widgets
                .iter_mut()
                .fold(
                    (0, available_space),
                    |(num_fill, fill_space), widget| match &widget.layout.sizing {
                        Sizing::Intrinsic => {
                            let size = widget.intrinsic_size();
                            widget.size = size;
                            (num_fill, reduce_fill_space(fill_space, size))
                        }
                        Sizing::Fill => (num_fill + 1, fill_space),
                    },
                );

        let size_per_widget = fill_space
            / match layout.direction {
                Direction::Horizontal => Size::new(num_fill, 1),
                Direction::Vertical => Size::new(1, num_fill),
            };

        let adjust_position = match &layout.direction {
            Direction::Horizontal => {
                |position: Position, size: Size| Position::new(position.x + size.width, position.y)
            }
            Direction::Vertical => {
                |position: Position, size: Size| Position::new(position.x, position.y + size.height)
            }
        };

        self.widgets
            .iter_mut()
            .fold(Position::zero(), |position, widget| {
                match &widget.layout.sizing {
                    Sizing::Intrinsic => {
                        widget.position = position;

                        adjust_position(position, widget.size)
                    }
                    Sizing::Fill => {
                        widget.size = size_per_widget;
                        widget.position = position;

                        adjust_position(position, size_per_widget)
                    }
                }
            });

        todo!()
    }
}

struct SizedView<'a> {
    sized: &'a [Widget<'a>],
    layout: Layout,
}
