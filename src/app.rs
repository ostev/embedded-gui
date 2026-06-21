use core::{fmt::Debug, marker::PhantomData};

use embedded_graphics::prelude::{Dimensions, DrawTarget};
use esp_println::println;
// use esp_println::println;

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
    pub const fn new() -> Self {
        Self {
            focus_key: None,
            effect: None,
            phantom: PhantomData,
        }
    }

    pub fn with_focus_key(mut self, focus_key: impl Into<FocusKey>) -> Self {
        self.focus_key = Some(focus_key.into());
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
    factory: Factory<Event, Msg, FocusKey>,
}

impl<Event, Msg, FocusKey: interactive::Key> InternalState<Event, Msg, FocusKey> {
    pub fn new(focus_key: FocusKey) -> Self {
        Self {
            factory: Factory::new(focus_key),
        }
    }
}

pub async fn dispatch<A: App>(
    app: &mut A,
    effect_context: &mut <A::Effect as Effect>::Context,
    internal_state: &mut InternalState<A::Event, A::Msg, A::FocusKey>,
    events: impl IntoIterator<Item = A::Event>,
) {
    for event in events {
        if let Some(msg) = internal_state.factory.dispatch(event) {
            dispatch_msg(app, effect_context, internal_state, msg).await;
        }
    }
}

pub async fn dispatch_msg<A: App>(
    app: &mut A,
    effect_context: &mut <A::Effect as Effect>::Context,
    internal_state: &mut InternalState<A::Event, A::Msg, A::FocusKey>,
    msg: A::Msg,
) {
    let mut next_msg = Some(msg);

    while let Some(msg) = next_msg.take() {
        println!(
            "Dispatching message: {:?}",
            core::any::type_name::<A::Msg>()
        );
        let change = app.update(msg);

        if let Some(key) = change.focus_key {
            internal_state.factory.set_focus_key(key);
            println!("Focus key changed to {:?}", key);
        } else {
            internal_state
                .factory
                .set_focus_key(internal_state.factory.focus_key());
            println!("Focus key unchanged");
        }

        next_msg = match change.effect {
            Some(effect) => effect.run(effect_context).await,
            None => None,
        };
    }
}

pub fn render<A: App>(
    app: &mut A,
    internal_state: &mut InternalState<A::Event, A::Msg, A::FocusKey>,
    display: &mut A::Target,
    is_init: bool,
) -> Result<(), <A::Target as DrawTarget>::Error>
where
    <A::Target as DrawTarget>::Color: Debug,
{
    // println!("Render actually");
    println!(
        "has focus changed? {}",
        internal_state.factory.has_focus_changed()
    );
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
