use alloc::vec::Vec;
use bumpalo::Bump;
use embedded_graphics::draw_target::DrawTarget;

use crate::{
    component::{Component, group::Group},
    draw::LocalTarget,
    event::{self, Handler},
    interactive::{self, FocusState},
    layout::{Direction, Sizing},
    position::Position,
    primitive::{Primitive, spacer::Spacer},
    signal::SignalRef,
    size::Size,
};

enum ComplexWidgetVariant<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg> {
    Component(
        &'a dyn Component<'a, T, FocusKey, Event, Msg>,
        &'a mut [Widget<'a, T, FocusKey, Event, Msg>],
    ),
    Primitive(&'a dyn Primitive<T>),
}

impl<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg>
    ComplexWidgetVariant<'a, T, FocusKey, Event, Msg>
{
    fn intrinsic_size(&self) -> Size {
        match self {
            ComplexWidgetVariant::Component(component, _) => component.intrinsic_size(),
            ComplexWidgetVariant::Primitive(primitive) => primitive.intrinsic_size(),
        }
    }

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

    fn primitive(bump: &'a Bump, primitive: impl Primitive<T> + 'a) -> Self {
        Self::Primitive(bump.alloc(primitive))
    }

    fn component<const N: usize>(
        bump: &'a Bump,
        component: impl Component<'a, T, FocusKey, Event, Msg> + 'a,
        children: [Widget<'a, T, FocusKey, Event, Msg>; N],
    ) -> Self {
        Self::Component(bump.alloc(component), bump.alloc(children))
    }

    fn component_ref(
        bump: &'a Bump,
        component: impl Component<'a, T, FocusKey, Event, Msg> + 'a,
        children: &'a mut [Widget<'a, T, FocusKey, Event, Msg>],
    ) -> Self {
        Self::Component(bump.alloc(component), children)
    }
}

struct ComplexWidget<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg> {
    inner: ComplexWidgetVariant<'a, T, FocusKey, Event, Msg>,
    sizing: Sizing,
    size: Size,
}

impl<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg>
    ComplexWidget<'a, T, FocusKey, Event, Msg>
{
    fn intrinsic_size(&self) -> Size {
        self.inner.intrinsic_size()
    }
}

struct InteractiveWidget<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg> {
    key: FocusKey,
    event_handler: event::Handler<Event, Msg>,
    view: &'a dyn Fn(Option<&FocusState>) -> ComplexWidget<'a, T, FocusKey, Event, Msg>,
    evaluated: Option<&'a mut ComplexWidget<'a, T, FocusKey, Event, Msg>>,
}

struct LayeredWidget<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg> {
    layers: &'a [Widget<'a, T, FocusKey, Event, Msg>],
}

enum WidgetVariant<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg> {
    Complex(ComplexWidget<'a, T, FocusKey, Event, Msg>),
    Interactive(InteractiveWidget<'a, T, FocusKey, Event, Msg>),
    // Layered(LayeredWidget<'a, T, FocusKey, Event, Msg>),
}

impl<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg>
    WidgetVariant<'a, T, FocusKey, Event, Msg>
{
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

    fn primitive(bump: &'a Bump, primitive: impl Primitive<T> + 'a, sizing: Sizing) -> Self {
        WidgetVariant::Complex(ComplexWidget {
            inner: ComplexWidgetVariant::primitive(bump, primitive),
            sizing,
            size: Size::zero(),
        })
    }

    fn component<const N: usize>(
        bump: &'a Bump,
        component: impl Component<'a, T, FocusKey, Event, Msg> + 'a,
        sizing: Sizing,
        children: [Widget<'a, T, FocusKey, Event, Msg>; N],
    ) -> Self {
        WidgetVariant::Complex(ComplexWidget {
            inner: ComplexWidgetVariant::component(bump, component, children),
            sizing,
            size: Size::zero(),
        })
    }

    fn component_ref(
        bump: &'a Bump,
        component: impl Component<'a, T, FocusKey, Event, Msg> + 'a,
        sizing: Sizing,
        children: &'a mut [Widget<'a, T, FocusKey, Event, Msg>],
    ) -> Self {
        WidgetVariant::Complex(ComplexWidget {
            inner: ComplexWidgetVariant::component_ref(bump, component, children),
            sizing,
            size: Size::zero(),
        })
    }
}

pub struct Widget<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg>(
    WidgetVariant<'a, T, FocusKey, Event, Msg>,
);

pub struct Factory {
    pub bump: Bump,
}

impl Factory {
    pub fn new() -> Factory {
        Factory { bump: Bump::new() }
    }

    pub fn component<'a, const N: usize, T: DrawTarget, FocusKey: interactive::Key, Event, Msg>(
        &'a self,
        sizing: Sizing,
        component: impl Component<'a, T, FocusKey, Event, Msg> + 'a,
        children: [Widget<'a, T, FocusKey, Event, Msg>; N],
    ) -> Widget<'a, T, FocusKey, Event, Msg> {
        Widget(WidgetVariant::component(
            &self.bump, component, sizing, children,
        ))
    }
    pub fn component_ref<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg>(
        &'a self,
        sizing: Sizing,
        component: impl Component<'a, T, FocusKey, Event, Msg> + 'a,
        children: &'a mut [Widget<'a, T, FocusKey, Event, Msg>],
    ) -> Widget<'a, T, FocusKey, Event, Msg> {
        Widget(WidgetVariant::component_ref(
            &self.bump, component, sizing, children,
        ))
    }

    pub fn primitive<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg>(
        &'a self,
        sizing: Sizing,
        primitive: impl Primitive<T> + 'a,
    ) -> Widget<'a, T, FocusKey, Event, Msg> {
        Widget(WidgetVariant::primitive(&self.bump, primitive, sizing))
    }

    pub fn view<'a, const N: usize, T: DrawTarget, FocusKey: interactive::Key, Event, Msg>(
        &'a self,
        direction: Direction,
        children: [Widget<'a, T, FocusKey, Event, Msg>; N],
    ) -> View<'a, T, FocusKey, Event, Msg> {
        View {
            widgets: self.bump.alloc(children),
            direction,
        }
    }

    pub fn view_ref<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg>(
        &self,
        direction: Direction,
        children: &'a mut [Widget<'a, T, FocusKey, Event, Msg>],
    ) -> View<'a, T, FocusKey, Event, Msg> {
        View {
            widgets: children,
            direction,
        }
    }

    pub fn spacer<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg>(
        &'a self,
    ) -> Widget<'a, T, FocusKey, Event, Msg> {
        self.primitive(Sizing::Fill, Spacer::zero())
    }

    pub fn group<'a, const N: usize, T: DrawTarget, FocusKey: interactive::Key, Event, Msg>(
        &'a self,
        direction: Direction,
        children: [Widget<'a, T, FocusKey, Event, Msg>; N],
    ) -> Widget<'a, T, FocusKey, Event, Msg> {
        self.component(
            Sizing::Fill,
            Group::zero(SignalRef::owned(direction)),
            children,
        )
    }

    pub fn group_ref<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg>(
        &'a self,
        direction: Direction,
        children: &'a mut [Widget<'a, T, FocusKey, Event, Msg>],
    ) -> Widget<'a, T, FocusKey, Event, Msg> {
        self.component_ref(
            Sizing::Fill,
            Group::zero(SignalRef::owned(direction)),
            children,
        )
    }

    pub fn centered<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg>(
        &'a self,
        direction: Direction,
        widget: Widget<'a, T, FocusKey, Event, Msg>,
    ) -> Widget<'a, T, FocusKey, Event, Msg> {
        self.group(direction, [self.spacer(), widget, self.spacer()])
    }

    pub fn middle<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg>(
        &'a self,
        widget: Widget<'a, T, FocusKey, Event, Msg>,
    ) -> Widget<'a, T, FocusKey, Event, Msg> {
        self.centered(
            Direction::Vertical,
            self.centered(Direction::Horizontal, widget),
        )
    }
}

pub struct View<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg> {
    widgets: &'a mut [Widget<'a, T, FocusKey, Event, Msg>],
    direction: Direction,
}

impl<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg> View<'a, T, FocusKey, Event, Msg> {
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
        focus_key: FocusKey,
        focus_state: &FocusState,
    ) -> Option<Size> {
        let reduce_fill_space = Self::reduce_fill_space(self.direction);

        let (num_fill, fill_space) = self.widgets.iter_mut().fold(
            (0, available_space),
            |(num_fill, fill_space), Widget(widget)| {
                let size_complex =
                    |complex: &mut ComplexWidget<'a, T, FocusKey, Event, Msg>| match complex.sizing
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
        handler_registry: &mut event::HandlerRegistry<FocusKey, Event, Msg>,
        origin: Position,
        available_space: Size,
        focus_key: FocusKey,
        focus_state: &FocusState,
        has_focus_changed: bool,
        target: &mut T,
        background_color: T::Color,
    ) -> Result<(), T::Error> {
        let size_per_widget = self
            .compute_size_per_widget(&factory.bump, available_space, focus_key, focus_state)
            .unwrap_or(Size::zero());
        let adjust_position = Self::adjust_position(self.direction);

        let mut position = Position::zero();

        let mut update_position =
            |complex: &mut ComplexWidget<'a, T, FocusKey, Event, Msg>| match complex.sizing {
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

                    // handler_registry.register(interactive.key, interactive.event_handler);

                    // focus_items.push(FocusItem {
                    //     position: widget_position,
                    //     key: interactive.key,
                    // });

                    (widget_position, complex)
                }
            };

            if has_changed {
                match &mut complex.inner {
                    ComplexWidgetVariant::Component(component, children) => {
                        let view = component.view(factory, children);

                        view.render(
                            factory,
                            handler_registry,
                            origin + widget_position,
                            complex.size,
                            focus_key,
                            focus_state,
                            has_focus_changed,
                            target,
                            background_color,
                        )?;

                        // focus_items.extend(view_focus_items);
                    }
                    ComplexWidgetVariant::Primitive(primitive) => {
                        match LocalTarget::try_new(target, origin + widget_position, complex.size) {
                            Some(mut local_target) => {
                                primitive.draw(&mut local_target)?;
                            }
                            None => {}
                        }
                    }
                }
            }
        }

        // Ok(focus_items)
        Ok(())
    }
}
