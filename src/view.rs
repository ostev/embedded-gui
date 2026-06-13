use core::{marker::PhantomData, mem::ManuallyDrop};

use alloc::boxed::Box;
use bumpalo::Bump;
use embedded_graphics::draw_target::DrawTarget;
// use esp_println::println;

use crate::{
    component::{Component, background::Background, group::Group},
    draw::LocalTarget,
    event::{Handler, HandlerRegistry},
    interactive::{self, FocusState},
    layout::{Direction, Sizing},
    position::Position,
    primitive::{Primitive, spacer::Spacer},
    signal::Signal,
    size::Size,
};

pub struct Children<
    'a,
    T: DrawTarget,
    Event,
    Msg,
    FocusKey: interactive::Key,
    AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    AnyPrimitive: Primitive<T>,
>(
    ManuallyDrop<
        bumpalo::boxed::Box<'a, [Widget<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>]>,
    >,
);

impl<
    'a,
    T: DrawTarget,
    Event,
    Msg,
    FocusKey: interactive::Key,
    AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    AnyPrimitive: Primitive<T>,
> Children<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>
{
    const fn new(
        children: bumpalo::boxed::Box<
            'a,
            [Widget<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>],
        >,
    ) -> Self {
        Self(ManuallyDrop::new(children))
    }
}

enum ComplexWidgetVariant<
    'a,
    T: DrawTarget,
    Event,
    Msg,
    FocusKey: interactive::Key,
    AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    AnyPrimitive: Primitive<T>,
> {
    Component(
        AnyComponent,
        // &'a mut [Widget<'a, T, Event, Msg, FocusKey, AnyComponent>],
        Children<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    ),
    Primitive(AnyPrimitive),
}

impl<
    'a,
    T: DrawTarget,
    Event,
    Msg,
    FocusKey: interactive::Key,
    AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    AnyPrimitive: Primitive<T>,
> ComplexWidgetVariant<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>
{
    fn intrinsic_size(&self) -> Size {
        match self {
            ComplexWidgetVariant::Component(component, _) => component.intrinsic_size(),
            ComplexWidgetVariant::Primitive(primitive) => primitive.intrinsic_size(),
        }
    }

    /// Determines whether a complex widget variant has changed and needs
    /// to be updated.

    fn has_changed(&self, focus_key: FocusKey, previous_focus_key: Option<FocusKey>) -> bool {
        match self {
            ComplexWidgetVariant::Component(component, children) => {
                component.has_changed()
                    || children.0.iter().any(|Widget { variant, .. }| {
                        variant.has_changed(focus_key, previous_focus_key)
                    })
            }
            ComplexWidgetVariant::Primitive(primitive) => primitive.has_changed(),
        }
    }

    fn primitive<P: Primitive<T> + 'a>(bump: &'a Bump, primitive: P) -> Self
    where
        bumpalo::boxed::Box<'a, P>: Into<AnyPrimitive>,
    {
        Self::Primitive(bumpalo::boxed::Box::new_in(primitive, bump).into())
    }

    fn component<
        const N: usize,
        C: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive> + 'a,
    >(
        bump: &'a Bump,
        component: C,
        children: [Widget<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>; N],
    ) -> Self
    where
        bumpalo::boxed::Box<'a, C>: Into<AnyComponent>,
    {
        Self::Component(
            bumpalo::boxed::Box::new_in(component, bump).into(),
            Children::new(bumpalo::boxed::Box::new_in(children, bump).into()),
        )
    }

    fn component_ref<C: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive> + 'a>(
        bump: &'a Bump,
        component: C,
        children: Children<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    ) -> Self
    where
        bumpalo::boxed::Box<'a, C>: Into<AnyComponent>,
    {
        Self::Component(
            bumpalo::boxed::Box::new_in(component, bump).into(),
            children,
        )
    }
}

struct ComplexWidget<
    'a,
    T: DrawTarget,
    Event,
    Msg,
    FocusKey: interactive::Key,
    AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    AnyPrimitive: Primitive<T>,
> {
    inner: ComplexWidgetVariant<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    sizing: Sizing,
    size: Size,
}

impl<
    'a,
    T: DrawTarget,
    Event,
    Msg,
    FocusKey: interactive::Key,
    AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    AnyPrimitive: Primitive<T>,
> ComplexWidget<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>
{
    fn intrinsic_size(&self) -> Size {
        self.inner.intrinsic_size()
    }
}

struct InteractiveWidget<
    'a,
    T: DrawTarget,
    Event,
    Msg,
    FocusKey: interactive::Key,
    AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    AnyPrimitive: Primitive<T>,
> {
    key: FocusKey,

    contents:
        bumpalo::boxed::Box<'a, Widget<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>>,

    phantom: PhantomData<(Event, Msg)>,
}

struct LayeredWidget<
    'a,
    T: DrawTarget,
    Event,
    Msg,
    FocusKey: interactive::Key,
    AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    AnyPrimitive: Primitive<T>,
> {
    layers: &'a [Widget<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>],
}

enum WidgetVariant<
    'a,
    T: DrawTarget,
    Event,
    Msg,
    FocusKey: interactive::Key,
    AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    AnyPrimitive: Primitive<T>,
> {
    Complex(ComplexWidget<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>),
    Interactive(InteractiveWidget<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>),
    // Layered(LayeredWidget<'a, T, Event, Msg, FocusKey>),
}

impl<
    'a,
    T: DrawTarget,
    Event,
    Msg,
    FocusKey: interactive::Key,
    AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    AnyPrimitive: Primitive<T>,
> WidgetVariant<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>
{
    /// Returns the nested complex widget inside a widget variant
    fn complex(
        &mut self,
    ) -> &mut ComplexWidget<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive> {
        match self {
            WidgetVariant::Complex(complex) => complex,
            WidgetVariant::Interactive(interactive) => interactive.contents.variant.complex(),
        }
    }

    /// Determines whether a widget variant has changed and needs to be updated.
    fn has_changed(&self, focus_key: FocusKey, previous_focus_key: Option<FocusKey>) -> bool {
        match self {
            Self::Complex(complex) => complex.inner.has_changed(focus_key, previous_focus_key),
            Self::Interactive(interactive) => {
                let has_focus_changed = Some(focus_key) != previous_focus_key;
                (has_focus_changed
                    && (interactive.key == focus_key
                        || Some(interactive.key) == previous_focus_key))
                    || interactive
                        .contents
                        .variant
                        .has_changed(focus_key, previous_focus_key)
            } // Self::Layered(LayeredWidget { layers }) => layers
              //     .iter()
              //     .any(|Widget(widget)| unsafe { widget.has_changed(has_focus_changed) }),
        }
    }

    fn primitive<P: Primitive<T> + 'a>(bump: &'a Bump, primitive: P, sizing: Sizing) -> Self
    where
        bumpalo::boxed::Box<'a, P>: Into<AnyPrimitive>,
    {
        WidgetVariant::Complex(ComplexWidget {
            inner: ComplexWidgetVariant::primitive(bump, primitive),
            sizing,
            size: Size::zero(),
        })
    }

    fn component<
        const N: usize,
        C: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive> + 'a,
    >(
        bump: &'a Bump,
        component: C,
        sizing: Sizing,
        children: [Widget<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>; N],
    ) -> Self
    where
        bumpalo::boxed::Box<'a, C>: Into<AnyComponent>,
    {
        WidgetVariant::Complex(ComplexWidget {
            inner: ComplexWidgetVariant::component(bump, component, children),
            sizing,
            size: Size::zero(),
        })
    }

    fn component_ref<C: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive> + 'a>(
        bump: &'a Bump,
        component: C,
        sizing: Sizing,
        children: Children<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    ) -> Self
    where
        bumpalo::boxed::Box<'a, C>: Into<AnyComponent>,
    {
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
    //     view: impl FnOnce(Option<&FocusState>) -> Widget<'a, T, Event, Msg, FocusKey>,
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
        contents: bumpalo::boxed::Box<
            'a,
            Widget<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
        >,
    ) -> Self {
        WidgetVariant::Interactive(InteractiveWidget {
            key,
            // event_handler: Some(event::Handler {
            //     handler: event_handler,
            // }),
            contents,
            phantom: PhantomData,
        })
    }
}

pub struct Widget<
    'a,
    T: DrawTarget,
    Event,
    Msg,
    FocusKey: interactive::Key,
    AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    AnyPrimitive: Primitive<T>,
> {
    variant: WidgetVariant<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,

    /// This phantom data marker is required as `T` is only used recursively.
    phantom: PhantomData<T>,
}

impl<
    'a,
    T: DrawTarget,
    Event,
    Msg,
    FocusKey: interactive::Key,
    AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    AnyPrimitive: Primitive<T>,
> Widget<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>
{
    const fn new(
        variant: WidgetVariant<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    ) -> Self {
        Self {
            variant,
            phantom: PhantomData,
        }
    }
}

pub struct Factory<Event, GlobalMsg, GlobalFocusKey: interactive::Key> {
    pub bump: Bump,

    handler_registry: HandlerRegistry<GlobalFocusKey, Event, GlobalMsg>,

    focus_key: GlobalFocusKey,
    focus_state: FocusState,
}

impl<Event, GlobalMsg, GlobalFocusKey: interactive::Key> Factory<Event, GlobalMsg, GlobalFocusKey> {
    pub fn new(focus_key: GlobalFocusKey) -> Self {
        Self {
            bump: Bump::new(),
            focus_state: FocusState::Unfocused,
            focus_key,
            handler_registry: HandlerRegistry::new(),
        }
    }

    pub(crate) fn set_focus_key(&mut self, key: GlobalFocusKey) {
        self.focus_key = key;
    }

    pub(crate) fn set_focus_state(&mut self, state: FocusState) {
        self.focus_state = state;
    }

    pub(crate) fn dispatch(&self, event: Event) -> Option<GlobalMsg> {
        self.handler_registry.dispatch(&self.focus_key, event)
    }

    pub fn interactive<
        'a,
        T: DrawTarget,
        FocusKey: Into<GlobalFocusKey>,
        Msg: Into<GlobalMsg>,
        AnyComponent: Component<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
        AnyPrimitive: Primitive<T>,
    >(
        &'a self,
        key: FocusKey,
        event_handler: impl Fn(Event) -> Msg + 'static,
        view: impl FnOnce(
            Option<FocusState>,
        )
            -> Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
    ) -> Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive> {
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

        Widget::new(WidgetVariant::interactive_ref(
            global_key,
            bumpalo::boxed::Box::new_in(contents, &self.bump),
        ))
    }

    pub fn component<
        'a,
        const N: usize,
        T: DrawTarget,
        C: Component<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive> + 'a,
        AnyComponent: Component<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
        AnyPrimitive: Primitive<T>,
    >(
        &'a self,
        sizing: Sizing,
        component: C,
        children: [Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>; N],
    ) -> Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>
    where
        bumpalo::boxed::Box<'a, C>: Into<AnyComponent>,
    {
        Widget::new(WidgetVariant::component(
            &self.bump, component, sizing, children,
        ))
    }

    pub fn component_ref<
        'a,
        T: DrawTarget,
        C: Component<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive> + 'a,
        AnyComponent: Component<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
        AnyPrimitive: Primitive<T>,
    >(
        &'a self,
        sizing: Sizing,
        component: C,
        children: Children<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
    ) -> Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>
    where
        bumpalo::boxed::Box<'a, C>: Into<AnyComponent>,
    {
        Widget::new(WidgetVariant::component_ref(
            &self.bump, component, sizing, children,
        ))
    }

    pub fn primitive<
        'a,
        T: DrawTarget,
        AnyComponent: Component<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
        AnyPrimitive: Primitive<T>,
        P: Primitive<T> + 'a,
    >(
        &'a self,
        sizing: Sizing,
        primitive: P,
    ) -> Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>
    where
        bumpalo::boxed::Box<'a, P>: Into<AnyPrimitive>,
    {
        Widget::new(WidgetVariant::primitive(&self.bump, primitive, sizing))
    }

    pub fn view<
        'a,
        const N: usize,
        T: DrawTarget,
        AnyComponent: Component<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
        AnyPrimitive: Primitive<T>,
    >(
        &'a self,
        direction: Direction,
        children: [Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>; N],
    ) -> View<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive> {
        View {
            internals: ViewInternals {
                widgets: Children::new(bumpalo::boxed::Box::new_in(children, &self.bump).into()),
                direction,
                phantom: PhantomData,
                background: None,
            },
        }
    }

    pub fn view_ref<
        'a,
        T: DrawTarget,
        AnyComponent: Component<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
        AnyPrimitive: Primitive<T>,
    >(
        &'a self,
        direction: Direction,
        children: Children<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
    ) -> View<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive> {
        View {
            internals: ViewInternals {
                widgets: children,
                direction,
                phantom: PhantomData,
                background: None,
            },
        }
    }

    pub fn spacer<
        'a,
        T: DrawTarget,
        AnyComponent: Component<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
        AnyPrimitive: Primitive<T>,
    >(
        &'a self,
    ) -> Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>
    where
        bumpalo::boxed::Box<'a, Spacer>: Into<AnyPrimitive>,
    {
        self.primitive(Sizing::Fill, Spacer::zero())
    }

    pub fn group<
        'a,
        const N: usize,
        T: DrawTarget,
        AnyComponent: Component<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
        AnyPrimitive: Primitive<T>,
    >(
        &'a self,
        direction: Direction,
        children: [Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>; N],
    ) -> Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>
    where
        bumpalo::boxed::Box<'a, Group>: Into<AnyComponent>,
    {
        let component = Group::zero(Signal::constant(direction));
        self.component(Sizing::Fill, component, children)
    }

    pub fn group_ref<
        'a,
        T: DrawTarget,
        AnyComponent: Component<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
        AnyPrimitive: Primitive<T>,
    >(
        &'a self,
        direction: Direction,
        children: Children<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
    ) -> Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>
    where
        bumpalo::boxed::Box<'a, Group>: Into<AnyComponent>,
    {
        let component: Group = Group::zero(Signal::constant(direction));
        self.component_ref(Sizing::Fill, component, children)
    }

    pub fn centered<
        'a,
        T: DrawTarget,
        AnyComponent: Component<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
        AnyPrimitive: Primitive<T>,
    >(
        &'a self,
        direction: Direction,
        widget: Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
    ) -> Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>
    where
        bumpalo::boxed::Box<'a, Group>: Into<AnyComponent>,
        bumpalo::boxed::Box<'a, Spacer>: Into<AnyPrimitive>,
    {
        self.group(direction, [self.spacer(), widget, self.spacer()])
    }

    pub fn middle<
        'a,
        T: DrawTarget,
        AnyComponent: Component<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
        AnyPrimitive: Primitive<T>,
    >(
        &'a self,
        widget: Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
    ) -> Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>
    where
        bumpalo::boxed::Box<'a, Group>: Into<AnyComponent>,
        bumpalo::boxed::Box<'a, Spacer>: Into<AnyPrimitive>,
    {
        self.centered(
            Direction::Vertical,
            self.centered(Direction::Horizontal, widget),
        )
    }
    pub fn background<
        'a,
        T: DrawTarget,
        AnyComponent: Component<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
        AnyPrimitive: Primitive<T>,
        const N: usize,
    >(
        &'a self,
        sizing: Sizing,
        color: Signal<T::Color>,
        children: [Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>; N],
    ) -> Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>
    where
        bumpalo::boxed::Box<'a, Background<T::Color>>: Into<AnyComponent>,
    {
        self.component(sizing, Background { color }, children)
    }

    pub fn background_ref<
        'a,
        T: DrawTarget,
        AnyComponent: Component<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
        AnyPrimitive: Primitive<T>,
    >(
        &'a self,
        sizing: Sizing,
        color: Signal<T::Color>,
        children: Children<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
    ) -> Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>
    where
        bumpalo::boxed::Box<'a, Background<T::Color>>: Into<AnyComponent>,
    {
        self.component_ref(sizing, Background { color }, children)
    }
}

/// Represents a collection of widgets laid out in a set direction.
pub struct View<
    'a,
    T: DrawTarget,
    Event,
    Msg,
    FocusKey: interactive::Key,
    AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    AnyPrimitive: Primitive<T>,
> {
    internals:
        ViewInternals<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive, UnsizedViewStage>,
}

struct ViewInternals<
    'a,
    T: DrawTarget,
    Event,
    Msg,
    FocusKey: interactive::Key,
    AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    AnyPrimitive: Primitive<T>,
    Stage: ViewProcessingStage,
> {
    widgets: Children<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    direction: Direction,

    background: Option<T::Color>,

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
fn reduce_fill_space_constrained(direction: Direction) -> impl Fn(Size, u16) -> Size {
    match direction {
        Direction::Horizontal => |fill_space: Size, constraint: u16| {
            Size::new(
                fill_space.width.saturating_sub(constraint),
                fill_space.height,
            )
        },
        Direction::Vertical => |fill_space: Size, constraint: u16| {
            Size::new(
                fill_space.width,
                fill_space.height.saturating_sub(constraint),
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

impl<'a, T: DrawTarget, Event, Msg, FocusKey: interactive::Key, AnyComponent, AnyPrimitive>
    View<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>
where
    AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    AnyPrimitive: Primitive<T>,
{
    pub fn with_background(mut self, background: T::Color) -> Self {
        self.internals.background = Some(background);

        self
    }

    /// This performs the initial sizing pass on the view's widgets, returning the sized view as well as
    /// the size for each fill widget or [`None`] if there aren't any widgets.
    fn compute_size_per_widget(
        mut self,
        bump: &'a Bump,
        available_space: Size,
        focus_key: FocusKey,
    ) -> (
        ViewInternals<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive, SizedViewStage>,
        Option<Size>,
    ) {
        let reduce_fill_space = reduce_fill_space(self.internals.direction);
        let reduce_fill_space_constrained = reduce_fill_space_constrained(self.internals.direction);

        /// This function is called in a recursive fold to calculate the size of a widget,
        /// mutating the original widget to store this information.
        fn size_widget<
            'a,
            T: DrawTarget,
            Event,
            Msg,
            FocusKey: interactive::Key,
            AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
            AnyPrimitive: Primitive<T>,
        >(
            bump: &'a Bump,
            focus_key: FocusKey,
            direction: Direction,
            reduce_fill_space: &impl Fn(Size, Size) -> Size,
            reduce_fill_space_constrained: &impl Fn(Size, u16) -> Size,
            (num_fill, fill_space): (u16, Size),
            Widget { variant, .. }: &mut Widget<
                'a,
                T,
                Event,
                Msg,
                FocusKey,
                AnyComponent,
                AnyPrimitive,
            >,
        ) -> (u16, Size) {
            // println!("Size!!!!");
            let size_complex = |complex: &mut ComplexWidget<
                'a,
                T,
                Event,
                Msg,
                FocusKey,
                AnyComponent,
                AnyPrimitive,
            >| {
                match complex.sizing {
                    Sizing::Intrinsic => {
                        let size = complex.intrinsic_size();
                        complex.size = size;
                        (num_fill, reduce_fill_space(fill_space, size))
                    }
                    Sizing::Constrained(constraint) => {
                        complex.size = match direction {
                            Direction::Horizontal => Size::new(constraint, fill_space.height),
                            Direction::Vertical => Size::new(fill_space.width, constraint),
                        };

                        (
                            num_fill + 1,
                            reduce_fill_space_constrained(fill_space, constraint),
                        )
                    }
                    Sizing::Fill => (num_fill + 1, fill_space),
                }
            };

            match variant {
                WidgetVariant::Complex(complex) => {
                    let (num_fill, fill_space) = size_complex(complex);
                    (num_fill, fill_space)
                }
                WidgetVariant::Interactive(interactive) => {
                    let (num_fill, fill_space) = size_widget(
                        bump,
                        focus_key,
                        direction,
                        reduce_fill_space,
                        reduce_fill_space_constrained,
                        (num_fill, fill_space),
                        &mut interactive.contents,
                    );

                    (num_fill, fill_space)
                }
            }
        }

        let (num_fill, fill_space) = self.internals.widgets.0.iter_mut().fold(
            (0, available_space),
            |(num_fill, fill_space), widget| {
                size_widget(
                    bump,
                    focus_key,
                    self.internals.direction,
                    &reduce_fill_space,
                    &reduce_fill_space_constrained,
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
                background: self.internals.background,

                phantom: PhantomData,
            },
            size_per_widget,
        )
    }

    pub(crate) unsafe fn render(
        self,
        factory: &'a Factory<Event, Msg, FocusKey>,
        origin: Position,
        available_space: Size,
        focus_key: FocusKey,
        previous_focus_key: Option<FocusKey>,
        target: &mut T,
        background_color: T::Color,
        is_init: bool,
    ) -> Result<(), T::Error> {
        let (mut sized_view, size_per_widget_option) =
            self.compute_size_per_widget(&factory.bump, available_space, focus_key);
        let size_per_widget = size_per_widget_option.unwrap_or(Size::zero());

        let adjust_position = adjust_position(sized_view.direction);

        fn update_position<
            'a,
            T: DrawTarget,
            Event,
            Msg,
            FocusKey: interactive::Key,
            AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
            AnyPrimitive: Primitive<T>,
        >(
            Widget { variant, .. }: &mut Widget<
                'a,
                T,
                Event,
                Msg,
                FocusKey,
                AnyComponent,
                AnyPrimitive,
            >,
            adjust_position: impl Fn(Position, Size) -> Position,
            size_per_widget: Size,
            position: Position,
        ) -> Position {
            match variant {
                WidgetVariant::Complex(complex) => match complex.sizing {
                    Sizing::Intrinsic => adjust_position(position, complex.size),
                    Sizing::Fill => {
                        complex.size = size_per_widget;

                        adjust_position(position, complex.size)
                    }
                    Sizing::Constrained(_) => adjust_position(position, complex.size),
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

        for widget in (sized_view.widgets.0).iter_mut() {
            // println!("Widget!");

            let has_changed = widget.variant.has_changed(focus_key, previous_focus_key);

            let new_position = update_position(widget, &adjust_position, size_per_widget, position);

            let complex = widget.variant.complex();

            if is_init || has_changed {
                // println!("Widget has changed!");
                match &mut complex.inner {
                    ComplexWidgetVariant::Component(component, children) => {
                        let view = {
                            // We need an owned copy of the component's children, which we can't get without
                            // unsafe code since Bumpalo's `Box<'a, T>` type doesn't implement `IntoIter`.
                            let view = {
                                let children_owned = Children::new(unsafe {
                                    bumpalo::boxed::Box::from_raw(&mut **children.0)
                                });

                                component.view(factory, children_owned)
                            };

                            unsafe { ManuallyDrop::drop(&mut children.0) };

                            view
                        };

                        // Safety: child views are built from the same factory bump and live
                        // for the duration of this render pass.
                        unsafe {
                            view.render(
                                factory,
                                origin + position,
                                complex.size,
                                focus_key,
                                previous_focus_key,
                                target,
                                sized_view.background.unwrap_or(background_color),
                                is_init,
                            )?
                        };
                    }
                    ComplexWidgetVariant::Primitive(primitive) => {
                        // println!("Primitive!");
                        match LocalTarget::try_new(target, origin + position, complex.size) {
                            Some(mut local_target) => {
                                local_target
                                    .clear(sized_view.background.unwrap_or(background_color))?;
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
