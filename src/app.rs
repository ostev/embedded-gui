use core::marker::PhantomData;

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

pub trait App: State + Reactive {
    type Target: DrawTarget;
    type Msg;
    type Event;
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

    fn new() -> Self;

    fn initial_focus_key() -> Self::FocusKey;

    fn background_color() -> <Self::Target as DrawTarget>::Color;
    fn update(&mut self, msg: Self::Msg) -> Change<Self::Msg, Self::FocusKey, Self::Effect>;
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
}

pub struct Change<Msg, FocusKey: interactive::Key, E: effect::Effect<Msg = Msg>> {
    focus_key: Option<FocusKey>,
    focus_state: Option<FocusState>,
    effect: Option<E>,

    phantom: PhantomData<Msg>,
}

impl<Msg, FocusKey: interactive::Key, E: effect::Effect<Msg = Msg>> Default
    for Change<Msg, FocusKey, E>
{
    fn default() -> Self {
        Self::none()
    }
}

impl<Msg, FocusKey: interactive::Key, E: effect::Effect<Msg = Msg>> Change<Msg, FocusKey, E> {
    pub const fn none() -> Self {
        Self {
            focus_key: None,
            focus_state: None,
            effect: None,
            phantom: PhantomData,
        }
    }

    pub const fn with_focus_key(mut self, focus_key: FocusKey) -> Self {
        self.focus_key = Some(focus_key);
        self
    }

    pub const fn with_focus_state(mut self, focus_state: FocusState) -> Self {
        self.focus_state = Some(focus_state);
        self
    }

    pub fn with_effect(mut self, effect: E) -> Self {
        self.effect = Some(effect);
        self
    }
}

pub trait State {
    fn mark_resolved(&mut self);
}

pub struct InternalState<Event, Msg, FocusKey: interactive::Key> {
    current_focus_key: FocusKey,
    previous_focus_key: Option<FocusKey>,
    factory: Factory<Event, Msg, FocusKey>,
}

impl<Event, Msg, FocusKey: interactive::Key> InternalState<Event, Msg, FocusKey> {
    pub fn new(focus_key: FocusKey) -> Self {
        Self {
            current_focus_key: focus_key,
            previous_focus_key: None,
            factory: Factory::new(focus_key),
        }
    }
}

pub async fn dispatch<A: App>(
    app: &mut A,
    internal_state: &mut InternalState<A::Event, A::Msg, A::FocusKey>,
    events: impl IntoIterator<Item = A::Event>,
) {
    for event in events {
        if let Some(msg) = internal_state.factory.dispatch(event) {
            dispatch_msg(app, internal_state, msg).await;
        }
    }
}

async fn dispatch_msg<A: App>(
    app: &mut A,
    internal_state: &mut InternalState<A::Event, A::Msg, A::FocusKey>,
    msg: A::Msg,
) {
    let mut next_msg = Some(msg);

    while let Some(msg) = next_msg.take() {
        let change = app.update(msg);

        if let Some(key) = change.focus_key {
            internal_state.previous_focus_key = Some(internal_state.current_focus_key);
            internal_state.current_focus_key = key;

            internal_state.factory.set_focus_key(key);
        }

        if let Some(state) = change.focus_state {
            internal_state.factory.set_focus_state(state);
        }

        next_msg = match change.effect {
            Some(effect) => Some(effect.run().await),
            None => None,
        };
    }
}

pub fn render<A: App>(
    app: &mut A,
    internal_state: &mut InternalState<A::Event, A::Msg, A::FocusKey>,
    display: &mut A::Target,
) -> Result<(), <A::Target as DrawTarget>::Error> {
    if app.has_changed() {
        let view = app.view(&internal_state.factory);

        // Safety: the view is rendered immediately after being built from this factory.
        let output = unsafe {
            view.render(
                &internal_state.factory,
                Position::zero(),
                display.bounding_box().size.into(),
                internal_state.current_focus_key,
                internal_state.previous_focus_key,
                display,
                A::background_color(),
            )
        };

        internal_state.factory.bump.reset();

        app.mark_resolved();

        output
    } else {
        Ok(())
    }
}
