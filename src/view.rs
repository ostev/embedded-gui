use std::convert::Infallible;

use alloc::vec::Vec;
use bumpalo::Bump;
use embedded_graphics::prelude::PixelColor;

use crate::{
    background::{Background, Layer},
    component::{Component, group::Group},
    draw::{self, Framebuffer, LocalTarget},
    interactive::{self, FocusItem, FocusState},
    layout::{Direction, Sizing},
    position::Position,
    primitive::{Primitive, spacer::Spacer, text::Text},
    signal::{Reactive, SignalRef},
    size::Size,
};

enum ComplexWidgetVariant<'a, Color: PixelColor> {
    Component(&'a dyn Component<'a, Color>, &'a mut [Widget<'a, Color>]),
    Primitive(&'a dyn Primitive<Color>),
}

impl<'a, Color: PixelColor> ComplexWidgetVariant<'a, Color> {
    fn intrinsic_size(&self) -> Size {
        match self {
            ComplexWidgetVariant::Component(component, _) => component.intrinsic_size(),
            ComplexWidgetVariant::Primitive(primitive) => primitive.intrinsic_size(),
        }
    }
}

impl<'a, Color: PixelColor> ComplexWidgetVariant<'a, Color> {
    /// Determines whether a complex widget variant has changed and needs
    /// to be updated.
    ///
    /// ## Safety notes
    /// It is the caller's responsibility to ensure that this
    /// widget has been evaluated if it is interactive. If called after the
    /// initial sizing pass, this will be the case.
    unsafe fn has_changed(&self, has_focus_changed: bool) -> bool {
        match self {
            ComplexWidgetVariant::Component(component, children) => {
                component.has_changed()
                    || children
                        .iter()
                        .any(|Widget(widget)| unsafe { widget.has_changed(has_focus_changed) })
            }
            ComplexWidgetVariant::Primitive(primitive) => primitive.has_changed(),
        }
    }

    fn primitive(bump: &'a Bump, primitive: impl Primitive<Color> + 'a) -> Self {
        Self::Primitive(bump.alloc(primitive))
    }

    fn component<const N: usize>(
        bump: &'a Bump,
        component: impl Component<'a, Color> + 'a,
        children: [Widget<'a, Color>; N],
    ) -> Self {
        Self::Component(bump.alloc(component), bump.alloc(children))
    }

    fn component_ref(
        bump: &'a Bump,
        component: impl Component<'a, Color> + 'a,
        children: &'a mut [Widget<'a, Color>],
    ) -> Self {
        Self::Component(bump.alloc(component), children)
    }
}

struct ComplexWidget<'a, Color: PixelColor> {
    inner: ComplexWidgetVariant<'a, Color>,
    sizing: Sizing,
    size: Size,
}

impl<'a, Color: PixelColor> ComplexWidget<'a, Color> {
    fn intrinsic_size(&self) -> Size {
        self.inner.intrinsic_size()
    }
}

struct InteractiveWidget<'a, Color: PixelColor> {
    key: interactive::Key,
    view: &'a dyn Fn(Option<&FocusState>) -> ComplexWidget<'a, Color>,
    evaluated: Option<&'a mut ComplexWidget<'a, Color>>,
}

struct LayeredWidget<'a, Color: PixelColor> {
    layers: &'a [Widget<'a, Color>],
}

enum WidgetVariant<'a, Color: PixelColor> {
    Complex(ComplexWidget<'a, Color>),
    Interactive(InteractiveWidget<'a, Color>),
    // Layered(LayeredWidget<'a, Color>),
}

impl<'a, Color: PixelColor> WidgetVariant<'a, Color> {
    /// Determines whether a widget variant has changed and needs to be updated.
    ///
    /// ## Safety notes
    /// It is the caller's responsibility to ensure that this
    /// widget has been evaluated if it is interactive. If called after the
    /// initial sizing pass, this will be the case.
    unsafe fn has_changed(&self, has_focus_changed: bool) -> bool {
        match self {
            Self::Complex(complex) => unsafe { complex.inner.has_changed(has_focus_changed) },
            Self::Interactive(interactive) => {
                let complex = unsafe { interactive.evaluated.as_deref().unwrap_unchecked() };
                has_focus_changed || unsafe { complex.inner.has_changed(has_focus_changed) }
            } // Self::Layered(LayeredWidget { layers }) => layers
              //     .iter()
              //     .any(|Widget(widget)| unsafe { widget.has_changed(has_focus_changed) }),
        }
    }

    fn primitive(bump: &'a Bump, primitive: impl Primitive<Color> + 'a, sizing: Sizing) -> Self {
        WidgetVariant::Complex(ComplexWidget {
            inner: ComplexWidgetVariant::primitive(bump, primitive),
            sizing,
            size: Size::zero(),
        })
    }

    fn component<const N: usize>(
        bump: &'a Bump,
        component: impl Component<'a, Color> + 'a,
        sizing: Sizing,
        children: [Widget<'a, Color>; N],
    ) -> Self {
        WidgetVariant::Complex(ComplexWidget {
            inner: ComplexWidgetVariant::component(bump, component, children),
            sizing,
            size: Size::zero(),
        })
    }

    fn component_ref(
        bump: &'a Bump,
        component: impl Component<'a, Color> + 'a,
        sizing: Sizing,
        children: &'a mut [Widget<'a, Color>],
    ) -> Self {
        WidgetVariant::Complex(ComplexWidget {
            inner: ComplexWidgetVariant::component_ref(bump, component, children),
            sizing,
            size: Size::zero(),
        })
    }
}

pub struct Widget<'a, Color: PixelColor>(WidgetVariant<'a, Color>);

pub struct Factory {
    pub bump: Bump,
}
impl Factory {
    pub fn component<'a, const N: usize, Color: PixelColor>(
        &'a self,
        sizing: Sizing,
        component: impl Component<'a, Color> + 'a,
        children: [Widget<'a, Color>; N],
    ) -> Widget<'a, Color> {
        Widget(WidgetVariant::component(
            &self.bump, component, sizing, children,
        ))
    }
    pub fn component_ref<'a, Color: PixelColor>(
        &'a self,
        sizing: Sizing,
        component: impl Component<'a, Color> + 'a,
        children: &'a mut [Widget<'a, Color>],
    ) -> Widget<'a, Color> {
        Widget(WidgetVariant::component_ref(
            &self.bump, component, sizing, children,
        ))
    }

    pub fn primitive<'a, Color: PixelColor>(
        &'a self,
        sizing: Sizing,
        primitive: impl Primitive<Color> + 'a,
    ) -> Widget<'a, Color> {
        Widget(WidgetVariant::primitive(&self.bump, primitive, sizing))
    }

    pub fn view<'a, const N: usize, Color: PixelColor>(
        &'a self,
        direction: Direction,
        children: [Widget<'a, Color>; N],
    ) -> View<'a, Color> {
        View {
            widgets: self.bump.alloc(children),
            direction,
        }
    }

    pub fn view_ref<'a, Color: PixelColor>(
        &self,
        direction: Direction,
        children: &'a mut [Widget<'a, Color>],
    ) -> View<'a, Color> {
        View {
            widgets: children,
            direction,
        }
    }

    pub fn spacer<'a, Color: PixelColor>(&'a self) -> Widget<'a, Color> {
        self.primitive(Sizing::Fill, Spacer::zero())
    }

    pub fn group<'a, const N: usize, Color: PixelColor>(
        &'a self,
        direction: Direction,
        children: [Widget<'a, Color>; N],
    ) -> Widget<'a, Color> {
        self.component(
            Sizing::Fill,
            Group::zero(SignalRef::owned(direction)),
            children,
        )
    }

    pub fn group_ref<'a, Color: PixelColor>(
        &'a self,
        direction: Direction,
        children: &'a mut [Widget<'a, Color>],
    ) -> Widget<'a, Color> {
        self.component_ref(
            Sizing::Fill,
            Group::zero(SignalRef::owned(direction)),
            children,
        )
    }

    pub fn centered<'a, Color: PixelColor>(
        &'a self,
        direction: Direction,
        widget: Widget<'a, Color>,
    ) -> Widget<'a, Color> {
        self.group(direction, [self.spacer(), widget, self.spacer()])
    }

    pub fn middle<'a, Color: PixelColor>(&'a self, widget: Widget<'a, Color>) -> Widget<'a, Color> {
        self.centered(
            Direction::Vertical,
            self.centered(Direction::Horizontal, widget),
        )
    }
}

pub struct View<'a, Color: PixelColor> {
    widgets: &'a mut [Widget<'a, Color>],
    direction: Direction,
}

impl<'a, Color: PixelColor> View<'a, Color> {
    fn reduce_fill_space(direction: Direction) -> impl Fn(Size, Size) -> Size {
        match direction {
            Direction::Horizontal => |fill_space: Size, size: Size| {
                Size::new(
                    fill_space.width.saturating_sub(size.width),
                    fill_space.height,
                )
            },
            Direction::Vertical => |fill_space: Size, size: Size| {
                Size::new(
                    fill_space.width,
                    fill_space.height.saturating_sub(size.height),
                )
            },
        }
    }

    fn adjust_position(direction: Direction) -> impl Fn(Position, Size) -> Position {
        match direction {
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
        let reduce_fill_space = Self::reduce_fill_space(self.direction);

        let (num_fill, fill_space) = self.widgets.iter_mut().fold(
            (0, available_space),
            |(num_fill, fill_space), Widget(widget)| {
                let size_complex = |complex: &mut ComplexWidget<'a, Color>| match complex.sizing {
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
                    / match self.direction {
                        Direction::Horizontal => Size::new(num_fill, 1),
                        Direction::Vertical => Size::new(1, num_fill),
                    },
            )
        } else {
            None
        }
    }

    pub(crate) fn render(
        mut self,
        factory: &'a Factory,
        origin: Position,
        available_space: Size,
        focus_key: interactive::Key,
        focus_state: &FocusState,
        has_focus_changed: bool,
        target: &mut draw::Framebuffer<Color>,
        background_color: Color,
    ) -> Result<Vec<FocusItem>, Infallible> {
        let size_per_widget = self
            .compute_size_per_widget(&factory.bump, available_space, focus_key, focus_state)
            .unwrap_or(Size::zero());
        let adjust_position = Self::adjust_position(self.direction);

        let mut position = Position::zero();
        let mut focus_items = Vec::new();

        let mut update_position = |complex: &mut ComplexWidget<'a, Color>| match complex.sizing {
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
            // Safety notes: this *should* be safe since we have explicitly set the
            // evaluated view during the previous pass.
            let has_changed = unsafe { widget.has_changed(has_focus_changed) };

            let (widget_position, complex) = match widget {
                WidgetVariant::Complex(complex) => (update_position(complex), complex),
                WidgetVariant::Interactive(interactive) => {
                    // Safety notes: same as above
                    let complex =
                        unsafe { interactive.evaluated.as_deref_mut().unwrap_unchecked() };
                    let widget_position = update_position(complex);

                    focus_items.push(FocusItem {
                        position: widget_position,
                        key: interactive.key,
                    });

                    (widget_position, complex)
                }
            };

            if has_changed {
                match &mut complex.inner {
                    ComplexWidgetVariant::Component(component, children) => {
                        let view = component.view(factory, children);

                        let view_focus_items = view.render(
                            factory,
                            origin + widget_position,
                            complex.size,
                            focus_key,
                            focus_state,
                            has_focus_changed,
                            target,
                            background_color,
                        )?;

                        focus_items.extend(view_focus_items);
                    }
                    ComplexWidgetVariant::Primitive(primitive) => {
                        match LocalTarget::try_new(target, origin + widget_position, complex.size) {
                            Some(mut local_target) => {
                                primitive.draw(&mut local_target);
                            }
                            None => {}
                        }
                    }
                }
            }
        }

        Ok(focus_items)
    }
}
