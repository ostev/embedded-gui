use std::borrow::Cow;

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

#[derive(Reactive, State)]
struct MyApp {
    text: Signal<String>,
    font_style: Signal<MonoTextStyle<'static, Rgb888>>,
}

enum Msg {}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum FocusKey {
    None,
}

enum Event {}

type Display = SimulatorDisplay<Rgb888>;

#[derive(Reactive)]
#[any_component(target = Display, event = Event, msg = Msg, focus_key = FocusKey)]
enum AnyComponent<'a> {
    Button(Button<'a, Rgb888, String>),
}

#[derive(Reactive)]
#[any_primitive(target = Display)]
enum AnyPrimitive<'a> {
    Text(Text<'a, Rgb888, String>),
    Spacer(Spacer<'a>),
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
                        text: SignalRef::owned("Say hi!".into()),
                        font_style: self.font_style.to_ref(),
                        size: SignalRef::owned(Size::new(128, 32)),
                    },
                    [],
                ),
            ],
        )
    }
}

#[pollster::main]
async fn main() {
    let mut display =
        SimulatorDisplay::<Rgb888>::new(embedded_graphics::prelude::Size::new(400, 400));

    let output_settings = OutputSettingsBuilder::new().build();

    let mut app = MyApp::new();
    let mut internal_state = InternalState::new(FocusKey::None);

    let mut window = Window::new("Hello World", &output_settings);
    // window.set_max_fps(60);

    // app::start(app, &mut display, || todo!(), after_render).await;
    // loop {
    display.clear(MyApp::background_color());
    app::render(&mut app, &mut internal_state, &mut display);
    window.show_static(&display);
    // }
}
