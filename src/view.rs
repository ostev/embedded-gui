use core::marker::PhantomData;

use alloc::boxed::Box;
use bumpalo::Bump;
use embedded_graphics::draw_target::DrawTarget;

use crate::{
    component::{Component, group::Group},
    draw::LocalTarget,
    event::{self, Handler, HandlerRegistry},
    interactive::{self, FocusState},
    layout::{Direction, Sizing},
    position::Position,
    primitive::{Primitive, spacer::Spacer},
    signal::SignalRef,
    size::Size,
};

enum ComplexWidgetVariant<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg> {
    Component(
        &'a dyn Component<'a, T, Event, Msg, FocusKey>,
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

    fn has_changed(&self, focus_key: FocusKey, previous_focus_key: FocusKey) -> bool {
        match self {
            ComplexWidgetVariant::Component(component, children) => {
                component.has_changed()
                    || children
                        .iter()
                        .any(|Widget(widget)| widget.has_changed(focus_key, previous_focus_key))
            }
            ComplexWidgetVariant::Primitive(primitive) => primitive.has_changed(),
        }
    }

    fn primitive(bump: &'a Bump, primitive: impl Primitive<T> + 'a) -> Self {
        Self::Primitive(bump.alloc(primitive))
    }

    fn component<const N: usize>(
        bump: &'a Bump,
        component: impl Component<'a, T, Event, Msg, FocusKey> + 'a,
        children: [Widget<'a, T, FocusKey, Event, Msg>; N],
    ) -> Self {
        Self::Component(bump.alloc(component), bump.alloc(children))
    }

    fn component_ref(
        bump: &'a Bump,
        component: impl Component<'a, T, Event, Msg, FocusKey> + 'a,
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

    contents: &'a mut Widget<'a, T, FocusKey, Event, Msg>,
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
    /// Registers all the event handlers for a given widget, returning the complex widget
    /// inside it.
    ///
    /// ### Safety notes
    /// This method **must not** be called more than once.
    fn register_handlers(
        &mut self,
        registry: &mut HandlerRegistry<FocusKey, Event, Msg>,
    ) -> &mut ComplexWidget<'a, T, FocusKey, Event, Msg> {
        match self {
            WidgetVariant::Complex(complex) => complex,
            WidgetVariant::Interactive(interactive) => {
                interactive.contents.register_handlers(registry)
            }
        }
    }

    fn get_complex(&self) -> &ComplexWidget<'a, T, FocusKey, Event, Msg> {
        match self {
            WidgetVariant::Complex(complex) => complex,
            WidgetVariant::Interactive(interactive) => interactive.contents.0.get_complex(),
        }
    }

    /// Determines whether a widget variant has changed and needs to be updated.
    fn has_changed(&self, focus_key: FocusKey, previous_focus_key: FocusKey) -> bool {
        match self {
            Self::Complex(complex) => complex.inner.has_changed(focus_key, previous_focus_key),
            Self::Interactive(interactive) => {
                let has_focus_changed = focus_key != previous_focus_key;
                (has_focus_changed
                    && (interactive.key == focus_key || interactive.key == previous_focus_key))
                    || interactive
                        .contents
                        .has_changed(focus_key, previous_focus_key)
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
        component: impl Component<'a, T, Event, Msg, FocusKey> + 'a,
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
        component: impl Component<'a, T, Event, Msg, FocusKey> + 'a,
        sizing: Sizing,
        children: &'a mut [Widget<'a, T, FocusKey, Event, Msg>],
    ) -> Self {
        WidgetVariant::Complex(ComplexWidget {
            inner: ComplexWidgetVariant::component_ref(bump, component, children),
            sizing,
            size: Size::zero(),
        })
    }

    // fn interactive(
    //     bump: &'a Bump,
    //     key: FocusKey,
    //     event_handler: impl Fn(Event) -> Msg,
    //     view: impl FnOnce(Option<&FocusState>) -> Widget<'a, T, FocusKey, Event, Msg>,
    // ) -> Self {
    //     WidgetVariant::Interactive(InteractiveWidget {
    //         key,
    //         event_handler: Some(event::Handler {
    //             handler: Box::new(event_handler),
    //         }),
    //         view: Box::new(view),
    //         evaluated: None,
    //     })
    // }

    fn interactive_ref(
        key: FocusKey,
        contents: &'a mut Widget<'a, T, FocusKey, Event, Msg>,
    ) -> Self {
        WidgetVariant::Interactive(InteractiveWidget {
            key,
            // event_handler: Some(event::Handler {
            //     handler: event_handler,
            // }),
            contents,
        })
    }
}

pub struct Widget<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg>(
    WidgetVariant<'a, T, FocusKey, Event, Msg>,
);

impl<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg>
    Widget<'a, T, FocusKey, Event, Msg>
{
    fn has_changed(&self, focus_key: FocusKey, previous_focus_key: FocusKey) -> bool {
        self.0.has_changed(focus_key, previous_focus_key)
    }

    fn register_handlers(
        &mut self,
        registry: &mut HandlerRegistry<FocusKey, Event, Msg>,
    ) -> &mut ComplexWidget<'a, T, FocusKey, Event, Msg> {
        self.0.register_handlers(registry)
    }
}

pub trait Factory<FocusKey: interactive::Key, Event, Msg> {
    fn bump(&self) -> &Bump;

    // fn map<OtherMsg>(
    //     &mut self,
    //     msg: impl Fn(OtherMsg) -> Msg + 'static,
    // ) -> impl Factory<FocusKey, Event, OtherMsg>;

    // fn map_focus<OtherFocusKey: interactive::Key>(
    //     &mut self,
    //     key: impl Fn(OtherFocusKey) -> FocusKey + 'static,
    // ) -> impl Factory<OtherFocusKey, Event, Msg>;

    fn component<'a, const N: usize, T: DrawTarget>(
        &'a self,
        sizing: Sizing,
        component: impl Component<'a, T, Event, Msg, FocusKey> + 'a,
        children: [Widget<'a, T, FocusKey, Event, Msg>; N],
    ) -> Widget<'a, T, FocusKey, Event, Msg> {
        Widget(WidgetVariant::component(
            self.bump(),
            component,
            sizing,
            children,
        ))
    }

    fn component_ref<'a, T: DrawTarget>(
        &'a self,
        sizing: Sizing,
        component: impl Component<'a, T, Event, Msg, FocusKey> + 'a,
        children: &'a mut [Widget<'a, T, FocusKey, Event, Msg>],
    ) -> Widget<'a, T, FocusKey, Event, Msg> {
        Widget(WidgetVariant::component_ref(
            self.bump(),
            component,
            sizing,
            children,
        ))
    }

    fn primitive<'a, T: DrawTarget>(
        &'a self,
        sizing: Sizing,
        primitive: impl Primitive<T> + 'a,
    ) -> Widget<'a, T, FocusKey, Event, Msg> {
        Widget(WidgetVariant::primitive(self.bump(), primitive, sizing))
    }

    // fn interactive<'a, T: DrawTarget>(
    //     &'a mut self,
    //     key: FocusKey,
    //     event_handler: impl Fn(Event) -> Msg + 'static,
    //     view: impl FnOnce(Option<FocusState>) -> Widget<'a, T, FocusKey, Event, Msg>,
    // ) -> Widget<'a, T, FocusKey, Event, Msg>;

    fn view<'a, const N: usize, T: DrawTarget>(
        &'a self,
        direction: Direction,
        children: [Widget<'a, T, FocusKey, Event, Msg>; N],
    ) -> View<'a, T, FocusKey, Event, Msg> {
        View {
            internals: ViewInternals {
                widgets: self.bump().alloc(children),
                direction,
                phantom: PhantomData,
            },
        }
    }

    fn view_ref<'a, T: DrawTarget>(
        &self,
        direction: Direction,
        children: &'a mut [Widget<'a, T, FocusKey, Event, Msg>],
    ) -> View<'a, T, FocusKey, Event, Msg> {
        View {
            internals: ViewInternals {
                widgets: children,
                direction,
                phantom: PhantomData,
            },
        }
    }

    fn spacer<'a, T: DrawTarget>(&'a self) -> Widget<'a, T, FocusKey, Event, Msg> {
        self.primitive(Sizing::Fill, Spacer::zero())
    }

    fn group<'a, const N: usize, T: DrawTarget>(
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

    fn group_ref<'a, T: DrawTarget>(
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

    fn centered<'a, T: DrawTarget>(
        &'a self,
        direction: Direction,
        widget: Widget<'a, T, FocusKey, Event, Msg>,
    ) -> Widget<'a, T, FocusKey, Event, Msg> {
        self.group(direction, [self.spacer(), widget, self.spacer()])
    }

    fn middle<'a, T: DrawTarget>(
        &'a self,
        widget: Widget<'a, T, FocusKey, Event, Msg>,
    ) -> Widget<'a, T, FocusKey, Event, Msg> {
        self.centered(
            Direction::Vertical,
            self.centered(Direction::Horizontal, widget),
        )
    }
}

pub(crate) struct GlobalFactory<GlobalFocusKey: interactive::Key, Event, GlobalMsg> {
    pub bump: Bump,

    handler_registry: HandlerRegistry<GlobalFocusKey, Event, GlobalMsg>,

    focus_key: GlobalFocusKey,
    focus_state: FocusState,
}

impl<GlobalFocusKey: interactive::Key, Event, GlobalMsg>
    GlobalFactory<GlobalFocusKey, Event, GlobalMsg>
{
    pub fn new(focus_key: GlobalFocusKey) -> Self {
        Self {
            bump: Bump::new(),
            focus_state: FocusState::Unfocused,
            focus_key,
            handler_registry: HandlerRegistry::new(),
        }
    }

    pub fn set_focus(&mut self, key: GlobalFocusKey, state: FocusState) {
        self.focus_key = key;
        self.focus_state = state;
    }

    pub fn interactive<'b, T: DrawTarget, FocusKey: Into<GlobalFocusKey>, Msg: Into<GlobalMsg>>(
        &'b mut self,
        key: FocusKey,
        event_handler: impl Fn(Event) -> Msg + 'static,
        view: impl FnOnce(Option<FocusState>) -> Widget<'b, T, GlobalFocusKey, Event, GlobalMsg>,
    ) -> Widget<'b, T, GlobalFocusKey, Event, GlobalMsg> {
        let global_key: GlobalFocusKey = key.into();

        let state = if self.focus_key == global_key {
            Some(self.focus_state)
        } else {
            None
        };

        let contents = view(state);

        // Box the event_handler into a trait object so it can be moved into a 'static closure
        let mapped_handler = Handler::new(Box::new(move |event| event_handler(event).into()));
        self.handler_registry.register(global_key, mapped_handler);

        Widget(WidgetVariant::interactive_ref(
            global_key,
            self.bump.alloc(contents),
        ))
    }
}

impl<FocusKey: interactive::Key, Event, Msg> Factory<FocusKey, Event, Msg>
    for GlobalFactory<FocusKey, Event, Msg>
{
    #[inline(always)]
    fn bump(&self) -> &Bump {
        &self.bump
    }

    // #[inline]
    // fn map<OtherMsg>(
    //     &mut self,
    //     f: impl Fn(OtherMsg) -> Msg + 'static,
    // ) -> impl Factory<FocusKey, Event, OtherMsg> {
    //     LocalFactory {
    //         global: self,
    //         map_msg: Rc::new(f),
    //         map_focus: Rc::new(|key| key),
    //         phantom: PhantomData,
    //     }
    // }

    // #[inline]
    // fn map_focus<OtherFocus: interactive::Key>(
    //     &mut self,
    //     f: impl Fn(OtherFocus) -> FocusKey + 'static,
    // ) -> impl Factory<OtherFocus, Event, Msg> {
    //     LocalFactory {
    //         global: self,
    //         map_msg: Rc::new(|msg| msg),
    //         map_focus: Rc::new(f),
    //         phantom: PhantomData,
    //     }
    // }
}

// pub struct LocalFactory<
//     'a,
//     FocusKey: Into<GlobalFocusKey>,
//     Event,
//     Msg: Into<GlobalMsg>,
//     GlobalMsg,
//     GlobalFocusKey: interactive::Key,
// > {
//     /// A reference to the global factory, which contains the bump allocator and
//     /// handler registry
//     global: &'a mut GlobalFactory<GlobalFocusKey, Event, GlobalMsg>,
//     // /// Transforms the child views' messages into something that the global factory can understand.
//     // map_msg: Rc<dyn Fn(Msg) -> GlobalMsg>,
//     // /// Transforms the child view's focus keys into something that the global factory can understand.
//     // map_focus: Rc<dyn Fn(FocusKey) -> GlobalFocusKey>,
//     /// Since this struct doesn't actually contain a `FocusKey` or a `Msg`, we need
//     /// a phantom data marker.
//     phantom: PhantomData<(FocusKey, Msg)>,
// }

// impl<
//     'a,
//     FocusKey: Into<GlobalFocusKey>,
//     Event,
//     Msg: Into<GlobalMsg>,
//     GlobalMsg,
//     GlobalFocusKey: interactive::Key,
// > LocalFactory<'a, FocusKey, Event, Msg, GlobalMsg, GlobalFocusKey>
// {

// }

// impl<
//     'a,
//     FocusKey: Into<GlobalFocusKey>,
//     Event,
//     Msg: Into<GlobalMsg>,
//     GlobalMsg,
//     GlobalFocusKey: interactive::Key,
// > Factory<GlobalFocusKey, Event, Msg>
//     for LocalFactory<'a, FocusKey, Event, Msg, GlobalMsg, GlobalFocusKey>
// {
//     #[inline(always)]
//     fn bump(&self) -> &Bump {
//         &self.global.bump
//     }

// #[inline]
// fn map<OtherMsg>(
//     &mut self,
//     f: impl Fn(OtherMsg) -> Msg + 'static,
// ) -> impl Factory<FocusKey, Event, OtherMsg> {
//     let map_msg = self.map_msg.clone();
//     let map_focus = &self.map_focus;

//     LocalFactory {
//         global: self.global,
//         map_msg: Rc::new(move |msg| map_msg(f(msg))),
//         map_focus: move |key| map_focus(key),
//         phantom: PhantomData,
//     }
// }

// #[inline]
// fn map_focus<OtherFocusKey: interactive::Key>(
//     &mut self,
//     f: impl Fn(OtherFocusKey) -> FocusKey,
// ) -> impl Factory<OtherFocusKey, Event, Msg> {
//     let map_msg = self.map_msg.clone();
//     let map_focus = &self.map_focus;

//     LocalFactory {
//         global: self.global,
//         map_msg: Rc::new(move |msg| map_msg(msg)),
//         map_focus: move |key| map_focus(f(key)),
//         phantom: PhantomData,
//     }
// }
// }

/// Represents a collection of widgets laid out in a set direction.
pub struct View<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg> {
    internals: ViewInternals<'a, T, FocusKey, Event, Msg, UnsizedViewStage>,
}

struct ViewInternals<
    'a,
    T: DrawTarget,
    FocusKey: interactive::Key,
    Event,
    Msg,
    Stage: ViewProcessingStage,
> {
    widgets: &'a mut [Widget<'a, T, FocusKey, Event, Msg>],
    direction: Direction,

    phantom: PhantomData<Stage>,
}

struct UnsizedViewStage {}
struct SizedViewStage {}

trait ViewProcessingStage {}

impl ViewProcessingStage for UnsizedViewStage {}
impl ViewProcessingStage for SizedViewStage {}

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

impl<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg> View<'a, T, FocusKey, Event, Msg> {
    /// This performs the initial sizing pass on the view's widgets, returning the sized view as well as
    /// the size for each fill widget or [`None`] if there aren't any widgets.
    fn compute_size_per_widget(
        self,
        bump: &'a Bump,
        available_space: Size,
        focus_key: FocusKey,
    ) -> (
        ViewInternals<'a, T, FocusKey, Event, Msg, SizedViewStage>,
        Option<Size>,
    ) {
        let reduce_fill_space = reduce_fill_space(self.internals.direction);

        /// This function is called in a recursive fold to calculate the size of a widget,
        /// mutating the original widget to store this information.
        fn size_widget<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg>(
            bump: &'a Bump,
            focus_key: FocusKey,
            reduce_fill_space: &impl Fn(Size, Size) -> Size,
            (num_fill, fill_space): (u16, Size),
            Widget(widget): &mut Widget<'a, T, FocusKey, Event, Msg>,
        ) -> (u16, Size) {
            let size_complex =
                |complex: &mut ComplexWidget<'a, T, FocusKey, Event, Msg>| match complex.sizing {
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
                    let (num_fill, fill_space) = size_widget(
                        bump,
                        focus_key,
                        reduce_fill_space,
                        (num_fill, fill_space),
                        &mut interactive.contents,
                    );

                    (num_fill, fill_space)
                }
            }
        }

        let (num_fill, fill_space) = self.internals.widgets.iter_mut().fold(
            (0, available_space),
            |(num_fill, fill_space), widget| {
                size_widget(
                    bump,
                    focus_key,
                    &reduce_fill_space,
                    (num_fill, fill_space),
                    widget,
                )
            },
        );

        let size_per_widget = if num_fill > 0 {
            Some(
                fill_space
                    / match self.internals.direction {
                        Direction::Horizontal => Size::new(num_fill, 1),
                        Direction::Vertical => Size::new(1, num_fill),
                    },
            )
        } else {
            None
        };

        (
            ViewInternals {
                widgets: self.internals.widgets,
                direction: self.internals.direction,
                phantom: PhantomData,
            },
            size_per_widget,
        )
    }

    pub(crate) fn render(
        self,
        factory: &'a GlobalFactory<FocusKey, Event, Msg>,
        handler_registry: &mut event::HandlerRegistry<FocusKey, Event, Msg>,
        origin: Position,
        available_space: Size,
        focus_key: FocusKey,
        previous_focus_key: FocusKey,
        target: &mut T,
        background_color: T::Color,
    ) -> Result<(), T::Error> {
        let (sized_view, size_per_widget_option) =
            self.compute_size_per_widget(&factory.bump(), available_space, focus_key);
        let size_per_widget = size_per_widget_option.unwrap_or(Size::zero());

        let adjust_position = adjust_position(sized_view.direction);

        fn update_position<'a, T: DrawTarget, FocusKey: interactive::Key, Event, Msg>(
            Widget(widget): &mut Widget<'a, T, FocusKey, Event, Msg>,
            adjust_position: impl Fn(Position, Size) -> Position,
            size_per_widget: Size,
            position: Position,
        ) -> Position {
            match widget {
                WidgetVariant::Complex(complex) => match complex.sizing {
                    Sizing::Intrinsic => adjust_position(position, complex.size),
                    Sizing::Fill => {
                        complex.size = size_per_widget;

                        adjust_position(position, complex.size)
                    }
                },
                WidgetVariant::Interactive(interactive) => update_position(
                    &mut interactive.contents,
                    adjust_position,
                    size_per_widget,
                    position,
                ),
            }
        }

        let mut position = Position::zero();

        for widget in sized_view.widgets.into_iter() {
            // Safety notes: this *should* be safe since we have explicitly set the
            // evaluated view during the previous pass.
            let has_changed = widget.has_changed(focus_key, previous_focus_key);

            let new_position = update_position(widget, &adjust_position, size_per_widget, position);

            let complex = widget.register_handlers(handler_registry);

            if has_changed {
                match &mut complex.inner {
                    ComplexWidgetVariant::Component(component, children) => {
                        let view = component.view(factory, children);

                        view.render(
                            factory,
                            handler_registry,
                            origin + position,
                            complex.size,
                            focus_key,
                            previous_focus_key,
                            target,
                            background_color,
                        )?;
                    }
                    ComplexWidgetVariant::Primitive(primitive) => {
                        match LocalTarget::try_new(target, origin + position, complex.size) {
                            Some(mut local_target) => {
                                primitive.draw(&mut local_target)?;
                            }
                            None => {}
                        }
                    }
                }
            }

            position = new_position;
        }

        // Ok(focus_items)
        Ok(())
    }
}
