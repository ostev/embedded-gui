use alloc::vec::Vec;
use bumpalo::Bump;
use embedded_graphics::prelude::PixelColor;

use crate::{
    component::Component,
    draw::{self, LocalTarget},
    interactive::{self, FocusItem},
    layout::{Direction, Layout, Sizing},
    position::Position,
    primitive::Primitive,
    signal::Reactive,
    size::Size,
};

enum ComplexWidgetVariant<'a, FocusState, Color: PixelColor> {
    Component(
        &'a dyn Component<FocusState, Color>,
        &'a mut [Widget<'a, FocusState, Color>],
    ),
    Primitive(&'a dyn Primitive<Color>),
}

impl<'a, FocusState, Color: PixelColor> Reactive for ComplexWidgetVariant<'a, FocusState, Color> {
    fn has_changed(&self) -> bool {
        match self {
            ComplexWidgetVariant::Component(component, _) => component.has_changed(),
            ComplexWidgetVariant::Primitive(primitive) => primitive.has_changed(),
        }
    }
}

impl<'a, FocusState, Color: PixelColor> ComplexWidgetVariant<'a, FocusState, Color> {
    fn intrinsic_size(&self) -> Size {
        match self {
            ComplexWidgetVariant::Component(component, _) => component.intrinsic_size(),
            ComplexWidgetVariant::Primitive(primitive) => primitive.intrinsic_size(),
        }
    }
}

impl<'a, FocusState, Color: PixelColor> ComplexWidgetVariant<'a, FocusState, Color> {
    fn primitive(bump: &'a Bump, primitive: impl Primitive<Color> + 'a) -> Self {
        Self::Primitive(bump.alloc(primitive))
    }

    fn component<const N: usize>(
        bump: &'a Bump,
        component: impl Component<FocusState, Color> + 'a,
        children: [Widget<'a, FocusState, Color>; N],
    ) -> Self {
        Self::Component(bump.alloc(component), bump.alloc(children))
    }

    fn component_ref(
        bump: &'a Bump,
        component: impl Component<FocusState, Color> + 'a,
        children: &'a mut [Widget<'a, FocusState, Color>],
    ) -> Self {
        Self::Component(bump.alloc(component), children)
    }
}

struct ComplexWidget<'a, FocusState, Color: PixelColor> {
    inner: ComplexWidgetVariant<'a, FocusState, Color>,
    layout: Layout,
    size: Size,
}

impl<'a, FocusState, Color: PixelColor> ComplexWidget<'a, FocusState, Color> {
    fn intrinsic_size(&self) -> Size {
        self.inner.intrinsic_size()
    }
}

struct InteractiveWidget<'a, FocusState, Color: PixelColor> {
    key: interactive::Key,
    view: &'a dyn Fn(Option<&FocusState>) -> ComplexWidget<'a, FocusState, Color>,
    evaluated: Option<&'a mut ComplexWidget<'a, FocusState, Color>>,
}

enum WidgetVariant<'a, FocusState, Color: PixelColor> {
    Complex(ComplexWidget<'a, FocusState, Color>),
    Interactive(InteractiveWidget<'a, FocusState, Color>),
}

impl<'a, FocusState, Color: PixelColor> WidgetVariant<'a, FocusState, Color> {
    fn primitive(bump: &'a Bump, primitive: impl Primitive<Color> + 'a, layout: Layout) -> Self {
        WidgetVariant::Complex(ComplexWidget {
            inner: ComplexWidgetVariant::primitive(bump, primitive),
            layout,
            size: Size::zero(),
        })
    }

    fn component<const N: usize>(
        bump: &'a Bump,
        component: impl Component<FocusState, Color> + 'a,
        layout: Layout,
        children: [Widget<'a, FocusState, Color>; N],
    ) -> Self {
        WidgetVariant::Complex(ComplexWidget {
            inner: ComplexWidgetVariant::component(bump, component, children),
            layout,
            size: Size::zero(),
        })
    }
}

pub struct Widget<'a, FocusState, Color: PixelColor>(WidgetVariant<'a, FocusState, Color>);

pub struct Factory {
    pub bump: Bump,
}
impl Factory {
    pub fn component<'a, const N: usize, FocusState, Color: PixelColor>(
        &'a self,
        component: impl Component<FocusState, Color> + 'a,
        layout: Layout,
        children: [Widget<'a, FocusState, Color>; N],
    ) -> Widget<'a, FocusState, Color> {
        Widget(WidgetVariant::component(
            &self.bump, component, layout, children,
        ))
    }

    pub fn primitive<'a, FocusState, Color: PixelColor>(
        &'a self,
        primitive: impl Primitive<Color> + 'a,
        layout: Layout,
    ) -> Widget<'a, FocusState, Color> {
        Widget(WidgetVariant::primitive(&self.bump, primitive, layout))
    }

    pub fn view<'a, const N: usize, FocusState, Color: PixelColor>(
        &'a self,
        layout: Layout,
        children: [Widget<'a, FocusState, Color>; N],
    ) -> View<'a, FocusState, Color> {
        View {
            widgets: self.bump.alloc(children),
            layout,
        }
    }

    pub fn view_ref<'a, FocusState, Color: PixelColor>(
        &self,
        layout: Layout,
        children: &'a mut [Widget<'a, FocusState, Color>],
    ) -> View<'a, FocusState, Color> {
        View {
            widgets: children,
            layout,
        }
    }
}

pub struct View<'a, FocusState, Color: PixelColor> {
    widgets: &'a mut [Widget<'a, FocusState, Color>],
    layout: Layout,
}

impl<'a, FocusState, Color: PixelColor> View<'a, FocusState, Color> {
    fn reduce_fill_space(layout: &Layout) -> impl Fn(Size, Size) -> Size {
        match layout.direction {
            Direction::Horizontal => |fill_space: Size, size: Size| {
                Size::new(fill_space.width - size.width, fill_space.height)
            },
            Direction::Vertical => |fill_space: Size, size: Size| {
                Size::new(fill_space.width, fill_space.height - size.height)
            },
        }
    }

    fn adjust_position(layout: &Layout) -> impl Fn(Position, Size) -> Position {
        match layout.direction {
            Direction::Horizontal => {
                |position: Position, size: Size| Position::new(position.x + size.width, position.y)
            }
            Direction::Vertical => {
                |position: Position, size: Size| Position::new(position.x, position.y + size.height)
            }
        }
    }

    fn compute_size_per_widget(
        &mut self,
        bump: &'a Bump,
        available_space: Size,
        focus_key: interactive::Key,
        focus_state: &FocusState,
    ) -> Option<Size> {
        let reduce_fill_space = Self::reduce_fill_space(&self.layout);

        let (num_fill, fill_space) = self.widgets.iter_mut().fold(
            (0, available_space),
            |(num_fill, fill_space), Widget(widget)| {
                let size_complex =
                    |complex: &mut ComplexWidget<'a, FocusState, Color>| match &complex
                        .layout
                        .sizing
                    {
                        Sizing::Intrinsic => {
                            let size = complex.intrinsic_size();
                            complex.size = size;
                            (num_fill, reduce_fill_space(fill_space, size))
                        }
                        Sizing::Fill => (num_fill + 1, fill_space),
                    };

                match widget {
                    WidgetVariant::Complex(complex) => {
                        let (num_fill, fill_space) = size_complex(complex);
                        (num_fill, fill_space)
                    }
                    WidgetVariant::Interactive(interactive) => {
                        let state = if focus_key == interactive.key {
                            Some(focus_state)
                        } else {
                            None
                        };

                        let interior = bump.alloc((interactive.view)(state));
                        let (num_fill, fill_space) = size_complex(interior);
                        interactive.evaluated = Some(interior);

                        (num_fill, fill_space)
                    }
                }
            },
        );

        if num_fill > 0 {
            Some(
                fill_space
                    / match self.layout.direction {
                        Direction::Horizontal => Size::new(num_fill, 1),
                        Direction::Vertical => Size::new(1, num_fill),
                    },
            )
        } else {
            None
        }
    }

    pub(crate) fn render<T: embedded_graphics::draw_target::DrawTarget<Color = Color>>(
        mut self,
        factory: &'a Factory,
        background: Color,
        available_space: Size,
        focus_key: interactive::Key,
        focus_state: &FocusState,
        has_focus_changed: bool,
        target: &mut T,
    ) -> Result<SizedView<'a, FocusState, Color>, T::Error> {
        let size_per_widget = self
            .compute_size_per_widget(&factory.bump, available_space, focus_key, focus_state)
            .unwrap_or(Size::zero());
        let adjust_position = Self::adjust_position(&self.layout);

        let mut position = Position::zero();
        let mut interactives = Vec::new();

        let mut update_position =
            |complex: &mut ComplexWidget<'a, FocusState, Color>| match complex.layout.sizing {
                Sizing::Intrinsic => {
                    let current_position = position;
                    position = adjust_position(position, complex.size);
                    current_position
                }
                Sizing::Fill => {
                    complex.size = size_per_widget;

                    let current_position = position;
                    position = adjust_position(position, complex.size);
                    current_position
                }
            };

        for Widget(widget) in self.widgets.iter_mut() {
            let (has_changed, widget_position, complex) = match widget {
                WidgetVariant::Complex(complex) => (
                    complex.inner.has_changed(),
                    update_position(complex),
                    complex,
                ),
                WidgetVariant::Interactive(interactive) => {
                    // Safety notes: this *should* be safe since we have explicitly set the
                    // evaluated view during the previous pass.
                    let complex =
                        unsafe { interactive.evaluated.as_deref_mut().unwrap_unchecked() };
                    let widget_position = update_position(complex);

                    interactives.push(FocusItem {
                        position: widget_position,
                        key: interactive.key,
                    });

                    (
                        has_focus_changed || complex.inner.has_changed(),
                        widget_position,
                        complex,
                    )
                }
            };

            let framebuffer_arena = Bump::new();

            if has_changed {
                match LocalTarget::try_new(
                    &framebuffer_arena,
                    background,
                    widget_position,
                    complex.size,
                ) {
                    Some(mut local_target) => match &complex.inner {
                        ComplexWidgetVariant::Component(component, children) => {
                            let view = component.view(factory, children);
                            let sized = view.render(
                                factory,
                                background,
                                complex.size,
                                focus_key,
                                focus_state,
                                has_focus_changed,
                                target,
                            )?;

                            interactives.extend(sized.interactives);
                        }
                        ComplexWidgetVariant::Primitive(primitive) => {
                            primitive.draw(&mut local_target);
                            local_target.blit(target)?
                        }
                    },
                    None => {}
                }
            }
        }

        Ok(SizedView {
            widgets: self.widgets,
            layout: self.layout,
            interactives,
        })
    }
}

pub(crate) struct SizedView<'a, FocusState, Color: PixelColor> {
    widgets: &'a [Widget<'a, FocusState, Color>],
    layout: Layout,
    interactives: Vec<FocusItem>,
}
