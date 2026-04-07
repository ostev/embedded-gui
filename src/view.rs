use alloc::vec::Vec;
use bumpalo::Bump;

use crate::{
    component::Component,
    interactive,
    layout::{Direction, Layout, Position, Size, Sizing},
    primitive::Primitive,
};

enum WidgetInner<'a, FocusState> {
    Component(
        &'a dyn Component<'a, FocusState>,
        &'a mut [Widget<'a, FocusState>],
    ),
    Primitive(&'a dyn Primitive),
}

impl<'a, FocusState> WidgetInner<'a, FocusState> {
    fn intrinsic_size(&self) -> Size {
        match self {
            WidgetInner::Component(component, _) => component.intrinsic_size(),
            WidgetInner::Primitive(primitive) => primitive.intrinsic_size(),
        }
    }
}

impl<'a, FocusState> WidgetInner<'a, FocusState> {
    fn primitive(bump: &'a Bump, primitive: impl Primitive + 'a) -> Self {
        Self::Primitive(bump.alloc(primitive))
    }

    fn component<const N: usize>(
        bump: &'a Bump,
        component: impl Component<'a, FocusState> + 'a,
        children: [Widget<'a, FocusState>; N],
    ) -> Self {
        Self::Component(bump.alloc(component), bump.alloc(children))
    }

    fn component_ref(
        bump: &'a Bump,
        component: impl Component<'a, FocusState> + 'a,
        children: &'a mut [Widget<'a, FocusState>],
    ) -> Self {
        Self::Component(bump.alloc(component), children)
    }
}

struct ComplexWidget<'a, FocusState> {
    inner: WidgetInner<'a, FocusState>,
    layout: Layout,
    size: Size,
    position: Position,
}

impl<'a, FocusState> ComplexWidget<'a, FocusState> {
    pub fn intrinsic_size(&self) -> Size {
        self.inner.intrinsic_size()
    }
}

struct InteractiveWidget<'a, FocusState> {
    key: interactive::Key,
    view: &'a dyn Fn(Option<&FocusState>) -> ComplexWidget<'a, FocusState>,
    evaluated: Option<&'a mut ComplexWidget<'a, FocusState>>,
}

pub enum Widget<'a, FocusState> {
    Complex(ComplexWidget<'a, FocusState>),
    Interactive(InteractiveWidget<'a, FocusState>),
}

impl<'a, FocusState> Widget<'a, FocusState> {
    fn primitive(bump: &'a Bump, primitive: impl Primitive + 'a, layout: Layout) -> Self {
        Widget::Complex(ComplexWidget {
            inner: WidgetInner::primitive(bump, primitive),
            layout,
            size: Size::zero(),
            position: Position::zero(),
        })
    }

    fn component<const N: usize>(
        bump: &'a Bump,
        component: impl Component<'a, FocusState> + 'a,
        layout: Layout,
        children: [Widget<'a, FocusState>; N],
    ) -> Self {
        Widget::Complex(ComplexWidget {
            inner: WidgetInner::component(bump, component, children),
            layout,
            size: Size::zero(),
            position: Position::zero(),
        })
    }
}

pub struct Factory<'a> {
    pub bump: &'a Bump,
}
impl<'a> Factory<'a> {
    pub fn component<const N: usize, FocusState>(
        &self,
        component: impl Component<'a, FocusState> + 'a,
        layout: Layout,
        children: [Widget<'a, FocusState>; N],
    ) -> Widget<'a, FocusState> {
        Widget::component(self.bump, component, layout, children)
    }

    pub fn primitive<FocusState>(
        &self,
        primitive: impl Primitive + 'a,
        layout: Layout,
    ) -> Widget<'a, FocusState> {
        Widget::primitive(self.bump, primitive, layout)
    }

    pub fn view<const N: usize, FocusState>(
        &self,
        children: [Widget<'a, FocusState>; N],
    ) -> View<'a, FocusState> {
        View {
            widgets: self.bump.alloc(children),
        }
    }

    pub fn view_ref<FocusState>(
        &self,
        children: &'a mut [Widget<'a, FocusState>],
    ) -> View<'a, FocusState> {
        View { widgets: children }
    }
}

pub struct View<'a, FocusState> {
    widgets: &'a mut [Widget<'a, FocusState>],
}

impl<'a, FocusState> View<'a, FocusState> {
    fn compute_size(
        self,
        bump: &'a Bump,
        layout: Layout,
        available_space: Size,
        focus_key: interactive::Key,
        focus_state: FocusState,
    ) -> SizedView<'a, FocusState> {
        let reduce_fill_space = match layout.direction {
            Direction::Horizontal => |fill_space: Size, size: Size| {
                Size::new(fill_space.width - size.width, fill_space.height)
            },
            Direction::Vertical => |fill_space: Size, size: Size| {
                Size::new(fill_space.width, fill_space.height - size.height)
            },
        };

        let (num_fill, fill_space, focus_order) = self.widgets.iter_mut().fold(
            (0, available_space, Vec::new()),
            |(num_fill, fill_space, mut focus_order), widget| {
                let size_complex =
                    |complex: &mut ComplexWidget<'a, FocusState>| match &complex.layout.sizing {
                        Sizing::Intrinsic => {
                            let size = complex.intrinsic_size();
                            complex.size = size;
                            (num_fill, reduce_fill_space(fill_space, size))
                        }
                        Sizing::Fill => (num_fill + 1, fill_space),
                    };

                match widget {
                    Widget::Complex(complex) => {
                        let (num_fill, fill_space) = size_complex(complex);
                        (num_fill, fill_space, focus_order)
                    }
                    Widget::Interactive(interactive) => {
                        let state = if focus_key == interactive.key {
                            Some(&focus_state)
                        } else {
                            None
                        };

                        let interior = bump.alloc((interactive.view)(state));
                        let (num_fill, fill_space) = size_complex(interior);
                        interactive.evaluated = Some(interior);

                        focus_order.push(interactive.key);

                        (num_fill, fill_space, focus_order)
                    }
                }
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
                let set_position =
                    |complex: &mut ComplexWidget<'a, FocusState>| match complex.layout.sizing {
                        Sizing::Intrinsic => {
                            complex.position = position;

                            adjust_position(position, complex.size)
                        }
                        Sizing::Fill => {
                            complex.size = size_per_widget;
                            complex.position = position;

                            adjust_position(position, size_per_widget)
                        }
                    };

                match widget {
                    Widget::Complex(complex) => set_position(complex),
                    Widget::Interactive(interactive) => {
                        // Safety notes: this *should* be safe since we have explicitly set the
                        // evaluated view during the previous pass.
                        let complex = unsafe { interactive.evaluated.as_mut().unwrap_unchecked() };
                        set_position(complex)
                    }
                }
            });

        SizedView {
            sized: self.widgets,
            layout: layout,
            focus_order,
        }
    }
}

struct SizedView<'a, FocusState> {
    sized: &'a [Widget<'a, FocusState>],
    layout: Layout,
    focus_order: Vec<interactive::Key>,
}
