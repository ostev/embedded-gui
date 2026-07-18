use core::{fmt::Debug, marker::PhantomData};

use embedded_graphics::prelude::{Dimensions, DrawTarget};

use crate::{
    component::Component,
    effect::{self, Effect},
    interactive::{self, FocusState},
    position::Position,
    primitive::Primitive,
    signal::Reactive,
    view::{self, Factory, View},
};

pub use embedded_gui_macros::State;

/// Top-level trait for an embedded-gui application.
///
/// The architecture is similar to Elm's Model-View-Update:
/// - [`App::new`] creates the initial state.
/// - [`App::update`] processes messages and returns [`Change`]s.
/// - [`App::view`] builds the view tree from the current state.
pub trait App: State + Reactive {
    type Target: DrawTarget;
    type Msg;
    type Event: Copy;
    type FocusKey: interactive::Key;
    type AnyComponent<'a>: Component<
            'a,
            Self::Target,
            Self::Event,
            Self::Msg,
            Self::FocusKey,
            Self::AnyComponent<'a>,
            Self::AnyPrimitive<'a>,
        >
    where
        Self: 'a;
    type AnyPrimitive<'a>: Primitive<Self::Target>
    where
        Self: 'a;

    type Effect: effect::Effect<Msg = Self::Msg>;

    /// Creates the initial application state.
    fn new() -> Self;

    /// Returns the focus key that should be active when the app starts.
    fn initial_focus_key() -> Self::FocusKey;

    /// The background color used to clear the display before each frame.
    fn background_color() -> <Self::Target as DrawTarget>::Color;

    /// Processes a message and returns a [`Change`] describing any requested
    /// focus change or side-effect.
    fn update(&mut self, msg: Self::Msg) -> Change<Self::Msg, Self::FocusKey, Self::Effect>;
    /// Builds the view tree for the current state using the given [`Factory`].
    fn view<'a>(
        &'a self,
        v: &'a view::Factory<Self::Event, Self::Msg, Self::FocusKey>,
    ) -> View<
        'a,
        Self::Target,
        Self::Event,
        Self::Msg,
        Self::FocusKey,
        Self::AnyComponent<'a>,
        Self::AnyPrimitive<'a>,
    >;

    /// Handler that receives every event before it is dispatched
    /// to the focused widget. Return `Some(msg)` to send a message.
    fn global_event_handler(&self, event: Self::Event) -> Option<Self::Msg>;
}

/// Describes side-effects and focus changes resulting from a message update.
pub struct Change<Msg, FocusKey: interactive::Key, E: effect::Effect<Msg = Msg>> {
    focus_key: Option<FocusKey>,
    effect: Option<E>,

    phantom: PhantomData<Msg>,
}

impl<Msg, FocusKey: interactive::Key, E: effect::Effect<Msg = Msg>> Default
    for Change<Msg, FocusKey, E>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Msg, FocusKey: interactive::Key, E: effect::Effect<Msg = Msg>> Change<Msg, FocusKey, E> {
    /// Creates a [`Change`] with no focus change and no effect.
    pub const fn new() -> Self {
        Self {
            focus_key: None,
            effect: None,
            phantom: PhantomData,
        }
    }

    /// Sets the focus key that should be active after this change.
    pub fn with_focus_key(mut self, focus_key: impl Into<FocusKey>) -> Self {
        self.focus_key = Some(focus_key.into());
        self
    }

    /// Sets the focus key only if `focus_key` is `Some`.
    pub fn with_focus_key_if_present(mut self, focus_key: Option<impl Into<FocusKey>>) -> Self {
        if let Some(focus_key) = focus_key {
            self.focus_key = Some(focus_key.into());
        }
        self
    }

    /// Attaches an effect that will be run after the update completes.
    pub fn with_effect(mut self, effect: E) -> Self {
        self.effect = Some(effect);
        self
    }
}

/// Trait for types whose "changed" flag can be cleared.
///
/// You can automatically derive this with `#[derive(State)]`.
pub trait State {
    /// Resets all change flags.
    fn mark_resolved(&mut self);
}

/// Internal state managed by the framework for a running application.
///
/// Holds the [`Factory`] used to build views and track keyboard focus.
pub struct InternalState<Event, Msg, FocusKey: interactive::Key> {
    factory: Factory<Event, Msg, FocusKey>,
}

impl<Event, Msg, FocusKey: interactive::Key> InternalState<Event, Msg, FocusKey> {
    /// Creates a new `InternalState` with the given initial focus key.
    pub fn new(focus_key: FocusKey) -> Self {
        Self {
            factory: Factory::new(focus_key),
        }
    }
}

/// Dispatches a sequence of events to the application.
///
/// Each event is first offered to [`App::global_event_handler`]; if that
/// returns `None`, it is forwarded to the currently focused widget's handler.
pub async fn dispatch<A: App>(
    app: &mut A,
    effect_context: &mut <A::Effect as Effect>::Context,
    internal_state: &mut InternalState<A::Event, A::Msg, A::FocusKey>,
    events: impl IntoIterator<Item = A::Event>,
) {
    for event in events {
        let msg = app
            .global_event_handler(event)
            .or_else(|| internal_state.factory.dispatch(event));
        if let Some(msg) = msg {
            dispatch_msg(app, effect_context, internal_state, msg).await;
        }
    }
}

/// Dispatches a single message to the application and processes any
/// resulting effects recursively.
pub async fn dispatch_msg<A: App>(
    app: &mut A,
    effect_context: &mut <A::Effect as Effect>::Context,
    internal_state: &mut InternalState<A::Event, A::Msg, A::FocusKey>,
    msg: A::Msg,
) {
    let mut next_msg = Some(msg);

    while let Some(msg) = next_msg.take() {
        let change = app.update(msg);

        if let Some(key) = change.focus_key {
            internal_state.factory.set_focus_key(key);
        } else {
            internal_state
                .factory
                .set_focus_key(internal_state.factory.focus_key());
        }

        next_msg = match change.effect {
            Some(effect) => effect.run(effect_context).await,
            None => None,
        };
    }
}

/// Renders the current application state to the display.
///
/// The view tree is discarded after rendering, resetting the bump arena.
pub fn render<A: App>(
    app: &mut A,
    internal_state: &mut InternalState<A::Event, A::Msg, A::FocusKey>,
    display: &mut A::Target,
    is_init: bool,
) -> Result<(), <A::Target as DrawTarget>::Error>
where
    <A::Target as DrawTarget>::Color: Debug,
{
    if is_init || internal_state.factory.has_focus_changed() || app.has_changed() {
        let view = app.view(&internal_state.factory);

        // Safety: the view is rendered immediately after being built from this factory.
        let output = unsafe {
            view.render(
                &internal_state.factory,
                Position::zero(),
                display.bounding_box().size.into(),
                display,
                A::background_color(),
                is_init,
            )
        };

        internal_state.factory.bump.reset();

        app.mark_resolved();

        output
    } else {
        Ok(())
    }
}
