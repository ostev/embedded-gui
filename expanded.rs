#![feature(prelude_import)]
extern crate std;
use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::{MonoTextStyle, iso_8859_13::FONT_10X20},
    pixelcolor::{Rgb888, RgbColor},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use embedded_gui::{
    app::{self, App, InternalState, State},
    component::{any_component, button::Button},
    interactive::FocusState,
    layout::{Direction, IntrinsicSize, Sizing},
    primitive::{any_primitive, spacer::Spacer, text::Text},
    signal::{Reactive, Signal, SignalRef},
    size::Size,
};
#[prelude_import]
use std::prelude::rust_2024::*;
struct MyApp {
    text: Signal<String>,
    font_style: Signal<MonoTextStyle<'static, Rgb888>>,
}
impl ::embedded_gui::signal::Reactive for MyApp {
    fn has_changed(&self) -> bool {
        ::embedded_gui::signal::Reactive::has_changed(&self.text)
            & ::embedded_gui::signal::Reactive::has_changed(&self.font_style)
    }
}
impl ::embedded_gui::app::State for MyApp {
    fn mark_resolved(&mut self) {
        ::embedded_gui::app::State::mark_resolved(&mut self.text);
        ::embedded_gui::app::State::mark_resolved(&mut self.font_style);
    }
}
enum Msg {}
enum FocusKey {
    None,
}
#[automatically_derived]
#[doc(hidden)]
unsafe impl ::core::clone::TrivialClone for FocusKey {}
#[automatically_derived]
impl ::core::clone::Clone for FocusKey {
    #[inline]
    fn clone(&self) -> FocusKey {
        *self
    }
}
#[automatically_derived]
impl ::core::marker::Copy for FocusKey {}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for FocusKey {}
#[automatically_derived]
impl ::core::cmp::PartialEq for FocusKey {
    #[inline]
    fn eq(&self, other: &FocusKey) -> bool {
        true
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for FocusKey {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {}
}
#[automatically_derived]
impl ::core::fmt::Debug for FocusKey {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(f, "None")
    }
}
#[automatically_derived]
impl ::core::hash::Hash for FocusKey {
    #[inline]
    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {}
}
enum Event {}
type Display = SimulatorDisplay<Rgb888>;
enum AnyComponent<'a> {
    Button(::bumpalo::boxed::Box<'a, Button<'a, Rgb888>>),
}
impl<'a, AnyPrimitive>
    ::embedded_gui::component::Component<'a, Display, Event, Msg, FocusKey, Self, AnyPrimitive>
    for AnyComponent<'a>
where
    AnyPrimitive: ::embedded_gui::primitive::Primitive<Display>,
{
    fn view(
        &self,
        v: &'a ::embedded_gui::view::Factory<Event, Msg, FocusKey>,
        children: ::embedded_gui::view::Children<
            'a,
            Display,
            Event,
            Msg,
            FocusKey,
            Self,
            AnyPrimitive,
        >,
    ) -> ::embedded_gui::view::View<'a, Display, Event, Msg, FocusKey, Self, AnyPrimitive> {
        match self {
            AnyComponent::Button(reference) => reference.view(v, children),
        }
    }
}
impl<'a> ::embedded_gui::layout::IntrinsicSize for AnyComponent<'a> {
    #[inline]
    fn intrinsic_size(&self) -> ::embedded_gui::size::Size {
        match self {
            AnyComponent::Button(reference) => reference.intrinsic_size(),
        }
    }
}
impl<'a> From<::bumpalo::boxed::Box<'a, Button<'a, Rgb888>>> for AnyComponent<'a> {
    fn from(component: ::bumpalo::boxed::Box<'a, Button<'a, Rgb888>>) -> Self {
        AnyComponent::Button(component)
    }
}
impl<'a> ::embedded_gui::signal::Reactive for AnyComponent<'a> {
    fn has_changed(&self) -> bool {
        match self {
            AnyComponent::Button(field_0) => ::embedded_gui::signal::Reactive::has_changed(field_0),
            _ => false,
        }
    }
}
enum AnyPrimitive<'a> {
    Text(::bumpalo::boxed::Box<'a, Text<'a, Rgb888>>),
    Spacer(::bumpalo::boxed::Box<'a, Spacer<'a>>),
}
impl<'a> ::embedded_gui::primitive::Primitive<Display> for AnyPrimitive<'a> {
    fn draw(
        &self,
        target: &mut ::embedded_gui::draw::LocalTarget<Display>,
    ) -> Result<(), Display::Error> {
        match self {
            AnyPrimitive::Text(reference) => reference.draw(target),
            AnyPrimitive::Spacer(reference) => reference.draw(target),
        }
    }
}
impl<'a> ::embedded_gui::layout::IntrinsicSize for AnyPrimitive<'a> {
    #[inline]
    fn intrinsic_size(&self) -> ::embedded_gui::size::Size {
        match self {
            AnyPrimitive::Text(reference) => reference.intrinsic_size(),
            AnyPrimitive::Spacer(reference) => reference.intrinsic_size(),
        }
    }
}
impl<'a> From<::bumpalo::boxed::Box<'a, Text<'a, Rgb888>>> for AnyPrimitive<'a> {
    fn from(primitive: ::bumpalo::boxed::Box<'a, Text<'a, Rgb888>>) -> Self {
        AnyPrimitive::Text(primitive)
    }
}
impl<'a> From<::bumpalo::boxed::Box<'a, Spacer<'a>>> for AnyPrimitive<'a> {
    fn from(primitive: ::bumpalo::boxed::Box<'a, Spacer<'a>>) -> Self {
        AnyPrimitive::Spacer(primitive)
    }
}
impl<'a> ::embedded_gui::signal::Reactive for AnyPrimitive<'a> {
    fn has_changed(&self) -> bool {
        match self {
            AnyPrimitive::Text(field_0) => ::embedded_gui::signal::Reactive::has_changed(field_0),
            AnyPrimitive::Spacer(field_0) => ::embedded_gui::signal::Reactive::has_changed(field_0),
            _ => false,
        }
    }
}
impl App for MyApp {
    type Target = SimulatorDisplay<Rgb888>;
    type Msg = Msg;
    type Event = Event;
    type FocusKey = FocusKey;
    type AnyComponent<'a> = AnyComponent<'a>;
    type AnyPrimitive<'a> = AnyPrimitive<'a>;
    fn initial_focus_key() -> Self::FocusKey {
        FocusKey::None
    }
    fn background_color() -> Rgb888 {
        Rgb888::WHITE
    }
    fn new() -> Self {
        Self {
            text: Signal::new("Hi".to_string()),
            font_style: Signal::new(MonoTextStyle::new(&FONT_10X20, Rgb888::RED)),
        }
    }
    fn update(&mut self, msg: Self::Msg) -> Option<(FocusKey, FocusState)> {
        None
    }
    fn view<'a>(
        &'a self,
        v: &'a embedded_gui::view::Factory<Self::Event, Self::Msg, Self::FocusKey>,
    ) -> embedded_gui::view::View<
        'a,
        Self::Target,
        Self::Event,
        Self::Msg,
        Self::FocusKey,
        Self::AnyComponent<'a>,
        Self::AnyPrimitive<'a>,
    > {
        v.view(
            Direction::Horizontal,
            [
                v.spacer(),
                v.primitive(
                    Sizing::Intrinsic,
                    Text {
                        content: self.text.to_ref(),
                        font_style: self.font_style.to_ref(),
                    },
                ),
                v.spacer(),
                v.primitive(
                    Sizing::Intrinsic,
                    Text {
                        content: self.text.to_ref(),
                        font_style: self.font_style.to_ref(),
                    },
                ),
                v.component(
                    Sizing::Fill,
                    Button {
                        text: SignalRef::owned("Say hi!"),
                        font_style: self.font_style.to_ref(),
                        size: SignalRef::owned(Size::new(128, 32)),
                    },
                    [],
                ),
            ],
        )
    }
}
fn main() {
    ::pollster::block_on(async {
        {
            let mut display =
                SimulatorDisplay::<Rgb888>::new(embedded_graphics::prelude::Size::new(400, 400));
            let output_settings = OutputSettingsBuilder::new().build();
            let mut app = MyApp::new();
            let mut internal_state = InternalState::new(FocusKey::None);
            let mut window = Window::new("Hello World", &output_settings);
            display.clear(MyApp::background_color());
            app::render(&mut app, &mut internal_state, &mut display);
            window.show_static(&display);
        }
    })
}
