use core::{fmt::Debug, marker::PhantomData};

use alloc::boxed::Box;
use bumpalo::Bump;
use embedded_graphics::draw_target::DrawTarget;

use crate::{
    component::{Component, background::Background, group::Group},
    draw::LocalTarget,
    event::{Handler, HandlerRegistry},
    interactive::{self, FocusState},
    layout::{Direction, Sizing},
    position::Position,
    primitive::{Primitive, spacer::Spacer},
    signal::{Signal, Source},
    size::Size,
};

/// A boxed slice of child widgets, produced by components.
pub struct Children<
    'a,
    T: DrawTarget,
    Event,
    Msg,
    FocusKey: interactive::Key,
    AnyComponent: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    AnyPrimitive: Primitive<T>,
>(Box<[Widget<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>], &'a Bump>);

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
        children: Box<[Widget<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>], &'a Bump>,
    ) -> Self {
        Self(children)
    }
}

/// Represents either a [`Primitive`] or a [`Component`]
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
                    || (Some(focus_key) != previous_focus_key)
            }
            ComplexWidgetVariant::Primitive(primitive) => primitive.has_changed(),
        }
    }

    fn primitive<P: Primitive<T> + 'a>(bump: &'a Bump, primitive: P) -> Self
    where
        Box<P, &'a Bump>: Into<AnyPrimitive>,
    {
        Self::Primitive(Box::new_in(primitive, bump).into())
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
        Box<C, &'a Bump>: Into<AnyComponent>,
    {
        Self::Component(
            Box::new_in(component, bump).into(),
            Children::new(Box::new_in(children, bump)),
        )
    }

    fn component_ref<C: Component<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive> + 'a>(
        bump: &'a Bump,
        component: C,
        children: Children<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>,
    ) -> Self
    where
        Box<C, &'a Bump>: Into<AnyComponent>,
    {
        Self::Component(Box::new_in(component, bump).into(), children)
    }
}

/// Either a [`Primitive`] or a [`Component`]
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

/// Represents a widget that can be interacted with.
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

    contents: Box<Widget<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>, &'a Bump>,

    phantom: PhantomData<(Event, Msg)>,
}

/// Internal type for widget variants, not exposed to the public API.
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
    fn complex(self) -> ComplexWidget<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive> {
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
                // println!(
                //     "has focus changed? {}, interactive key: {:?}, current focus key: {:?}, previous focus key: {:?}",
                //     has_focus_changed, interactive.key, focus_key, previous_focus_key
                // );
                let has_changed = has_focus_changed
                    || interactive
                        .contents
                        .variant
                        .has_changed(focus_key, previous_focus_key);
                // println!("interactive has changed? {}", has_changed);
                has_changed
            } // Self::Layered(LayeredWidget { layers }) => layers
              //     .iter()
              //     .any(|Widget(widget)| unsafe { widget.has_changed(has_focus_changed) }),
        }
    }

    fn primitive<P: Primitive<T> + 'a>(bump: &'a Bump, primitive: P, sizing: Sizing) -> Self
    where
        Box<P, &'a Bump>: Into<AnyPrimitive>,
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
        Box<C, &'a Bump>: Into<AnyComponent>,
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
        Box<C, &'a Bump>: Into<AnyComponent>,
    {
        WidgetVariant::Complex(ComplexWidget {
            inner: ComplexWidgetVariant::component_ref(bump, component, children),
            sizing,
            size: Size::zero(),
        })
    }

    fn interactive_ref(
        key: FocusKey,
        contents: Box<Widget<'a, T, Event, Msg, FocusKey, AnyComponent, AnyPrimitive>, &'a Bump>,
    ) -> Self {
        WidgetVariant::Interactive(InteractiveWidget {
            key,
            contents,
            phantom: PhantomData,
        })
    }
}

/// A widget node in the view tree.
///
/// Widgets are either [`Complex`](WidgetVariant::Complex) (a component or primitive)
/// or [`Interactive`](WidgetVariant::Interactive) (a focusable wrapper around another widget).
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

/// The main builder for constructing view trees.
///
/// Holds a bump arena for allocations and a registry of event handlers
/// keyed by focus key. All widgets, views, and children are created
/// through this factory.
pub struct Factory<Event, GlobalMsg, GlobalFocusKey: interactive::Key> {
    /// The bump allocator used for all view allocations.
    pub bump: Bump,

    handler_registry: HandlerRegistry<GlobalFocusKey, Event, GlobalMsg>,

    focus_key: GlobalFocusKey,
    previous_focus_key: Option<GlobalFocusKey>,
}

impl<Event, GlobalMsg, GlobalFocusKey: interactive::Key> Factory<Event, GlobalMsg, GlobalFocusKey> {
    /// Creates a new factory with the given initial focus key.
    pub fn new(focus_key: GlobalFocusKey) -> Self {
        Self {
            bump: Bump::new(),
            focus_key,
            previous_focus_key: None,
            handler_registry: HandlerRegistry::new(),
        }
    }

    /// Returns the current focus key
    pub(crate) fn focus_key(&self) -> GlobalFocusKey {
        self.focus_key
    }

    /// Updates the current focus key
    pub(crate) fn set_focus_key(&mut self, key: GlobalFocusKey) {
        self.previous_focus_key = Some(self.focus_key);
        self.focus_key = key;
    }

    /// Has the focus key changed?
    pub(crate) fn has_focus_changed(&self) -> bool {
        Some(self.focus_key) != self.previous_focus_key
    }

    /// Dispatch an event to the focused interactive widget.
    pub(crate) fn dispatch(&self, event: Event) -> Option<GlobalMsg> {
        self.handler_registry.dispatch(&self.focus_key, event)
    }

    /// Creates an interactive widget that can receive focus and handle events.
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
        event_handler: impl Fn(Event) -> Option<Msg> + 'static,
        view: impl FnOnce(
            Signal<FocusState>,
        )
            -> Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
    ) -> Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive> {
        let global_key: GlobalFocusKey = key.into();

        let has_changed = Some(self.focus_key) != self.previous_focus_key
            && (global_key == self.focus_key || Some(global_key) == self.previous_focus_key);
        let state = if self.focus_key == global_key {
            Source::custom(FocusState::Focused, has_changed).signal()
        } else {
            Source::custom(FocusState::Unfocused, has_changed).signal()
        };

        let contents = view(state);

        // Box the event_handler into a trait object so it can be moved into a 'static closure
        let mapped_handler = Handler::new(Box::new(move |event| {
            event_handler(event).map(|msg| msg.into())
        }));
        self.handler_registry.register(global_key, mapped_handler);

        Widget::new(WidgetVariant::interactive_ref(
            global_key,
            Box::new_in(contents, &self.bump),
        ))
    }

    /// Creates a widget from a component with the given sizing and children.
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
        Box<C, &'a Bump>: Into<AnyComponent>,
    {
        Widget::new(WidgetVariant::component(
            &self.bump, component, sizing, children,
        ))
    }

    /// Creates a widget from a component with pre-existing children.
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
        Box<C, &'a Bump>: Into<AnyComponent>,
    {
        Widget::new(WidgetVariant::component_ref(
            &self.bump, component, sizing, children,
        ))
    }

    /// Creates a leaf widget from a primitive.
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
        Box<P, &'a Bump>: Into<AnyPrimitive>,
    {
        Widget::new(WidgetVariant::primitive(&self.bump, primitive, sizing))
    }

    /// Creates a [`View`] that arranges children in the given direction.
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
                widgets: Children::new(Box::new_in(children, &self.bump)),
                direction,
                phantom: PhantomData,
                background: None,
            },
        }
    }

    /// Creates a [`View`] from pre-existing children.
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

    /// Creates a fill-sized spacer widget.
    pub fn spacer<
        'a,
        T: DrawTarget,
        AnyComponent: Component<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
        AnyPrimitive: Primitive<T>,
    >(
        &'a self,
    ) -> Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>
    where
        Box<Spacer, &'a Bump>: Into<AnyPrimitive>,
    {
        self.primitive(Sizing::Fill, Spacer::zero())
    }

    /// Creates spacer widget with set size.
    pub fn sized_spacer<
        'a,
        T: DrawTarget,
        AnyComponent: Component<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
        AnyPrimitive: Primitive<T>,
    >(
        &'a self,
        size: Signal<Size>,
    ) -> Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>
    where
        Box<Spacer, &'a Bump>: Into<AnyPrimitive>,
    {
        self.primitive(Sizing::Intrinsic, Spacer { size })
    }

    /// Creates a fill-sized group widget.
    pub fn group_fill<
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
        Box<Group, &'a Bump>: Into<AnyComponent>,
    {
        let component = Group::zero(Signal::constant(direction));
        self.component(Sizing::Fill, component, children)
    }

    /// Creates a group widget with the given sizing.
    pub fn group<
        'a,
        const N: usize,
        T: DrawTarget,
        AnyComponent: Component<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
        AnyPrimitive: Primitive<T>,
    >(
        &'a self,
        direction: Direction,
        sizing: Sizing,
        children: [Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>; N],
    ) -> Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>
    where
        Box<Group, &'a Bump>: Into<AnyComponent>,
    {
        let component = Group::zero(Signal::constant(direction));
        self.component(sizing, component, children)
    }

    /// Creates a group widget from pre-existing children.
    pub fn group_ref<
        'a,
        T: DrawTarget,
        AnyComponent: Component<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
        AnyPrimitive: Primitive<T>,
    >(
        &'a self,
        direction: Direction,
        sizing: Sizing,
        children: Children<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>,
    ) -> Widget<'a, T, Event, GlobalMsg, GlobalFocusKey, AnyComponent, AnyPrimitive>
    where
        Box<Group, &'a Bump>: Into<AnyComponent>,
    {
        let component: Group = Group::zero(Signal::constant(direction));
        self.component_ref(sizing, component, children)
    }

    /// Centers a widget horizontally or vertically using fill spacers.
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
        Box<Group, &'a Bump>: Into<AnyComponent>,
        Box<Spacer, &'a Bump>: Into<AnyPrimitive>,
    {
        self.group_fill(direction, [self.spacer(), widget, self.spacer()])
    }

    /// Centers a widget both horizontally and vertically.
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
        Box<Group, &'a Bump>: Into<AnyComponent>,
        Box<Spacer, &'a Bump>: Into<AnyPrimitive>,
    {
        self.centered(
            Direction::Vertical,
            self.centered(Direction::Horizontal, widget),
        )
    }
    /// Creates a background-colored group widget.
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
        Box<Background<T::Color>, &'a Bump>: Into<AnyComponent>,
        T::Color: Debug + 'a,
    {
        self.component(sizing, Background { color }, children)
    }

    /// Creates a background-colored group widget from pre-existing children.
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
        Box<Background<T::Color>, &'a Bump>: Into<AnyComponent>,
        T::Color: Debug + 'a,
    {
        self.component_ref(sizing, Background { color }, children)
    }
}

/// Represents a collection of widgets laid out in a set direction.
///
/// A `View` is the result of rendering a [`Component`].
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

/// Internal type for views not exposed to public API. It can be in one of two stages: unsized
/// (freshly built) and sized (after layout).
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

// Basically these act as type-level enums:

struct UnsizedViewStage {}
struct SizedViewStage {}

trait ViewProcessingStage {}

impl ViewProcessingStage for UnsizedViewStage {}
impl ViewProcessingStage for SizedViewStage {}

/// Return a function to reduce the available fill space in the specified direction
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

/// Return a function to reduce the available fill space in the specified with a constraint.
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

/// Returns a function to adjust the current position to after the widget being drawn
/// in the specified direction.
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
    T::Color: Debug,
{
    /// Sets an optional background color for this view's rendering area.
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

        /// Called in a recursive fold to calculate the size of each widget. This function
        /// mutates the original widget to store this information.
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
                        // println!("Intrinsic size: {:?}", size);
                        complex.size = size;
                        (num_fill, reduce_fill_space(fill_space, size))
                    }
                    Sizing::Constrained(constraint) => {
                        complex.size = match direction {
                            Direction::Horizontal => Size::new(constraint, fill_space.height),
                            Direction::Vertical => Size::new(fill_space.width, constraint),
                        };

                        (
                            num_fill,
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

    pub(crate) fn render(
        self,
        factory: &'a Factory<Event, Msg, FocusKey>,
        origin: Position,
        available_space: Size,
        target: &mut T,
        background_color: T::Color,
        is_init: bool,
    ) -> Result<(), T::Error> {
        let (mut sized_view, size_per_widget_option) =
            self.compute_size_per_widget(&factory.bump, available_space, factory.focus_key);
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

        for mut widget in sized_view.widgets.0 {
            // println!("Widget!");

            let has_changed = widget
                .variant
                .has_changed(factory.focus_key, factory.previous_focus_key);

            let new_position =
                update_position(&mut widget, &adjust_position, size_per_widget, position);

            let complex = widget.variant.complex();

            if is_init || has_changed {
                // The widget's change, so we'll re-render it!
                match complex.inner {
                    ComplexWidgetVariant::Component(component, children) => {
                        let view = component.view(factory, children);

                        view.render(
                            factory,
                            origin + position,
                            complex.size,
                            target,
                            sized_view.background.unwrap_or(background_color),
                            is_init,
                        )?;
                    }
                    ComplexWidgetVariant::Primitive(primitive) => {
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
