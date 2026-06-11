use std::borrow::Cow;

use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::{MonoTextStyle, MonoTextStyleBuilder, iso_8859_13::FONT_10X20},
    pixelcolor::{Rgb565, Rgb888, RgbColor},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use embedded_gui::{
    app::{self, App, Change, InternalState, State},
    component::{any_component, button::Button, group::Group},
    interactive::FocusState,
    layout::{Direction, IntrinsicSize, Sizing},
    primitive::{any_primitive, spacer::Spacer, text::Text},
    signal::{Reactive, Signal, SignalRef, Source},
    size::Size,
};

#[derive(Reactive, State)]
struct MyApp {
    text: Source<String>,
    font_style: Source<MonoTextStyle<'static, Color>>,
}

enum Msg {
    Hi,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum FocusKey {
    None,
}

enum Event {}

type Color = Rgb565;

type Display = SimulatorDisplay<Color>;

#[derive(Reactive)]
#[any_component(target = Display, event = Event, msg = Msg, focus_key = FocusKey)]
enum AnyComponent<'a> {
    Button(Button<'a, Color, &'static str>),
    Group(Group),
}

#[derive(Reactive)]
#[any_primitive(target = Display)]
enum AnyPrimitive<'a> {
    TextStatic(Text<'a, Color, &'static str>),
    TextOwned(Text<'a, Color, String>),
    Spacer(Spacer),
}

enum Effect {}
impl embedded_gui::effect::Effect for Effect {
    type Msg = Msg;

    type Context = ();

    fn run(self, context: &mut Self::Context) -> impl Future<Output = Self::Msg> {
        core::future::ready(Msg::Hi)
    }
}

impl App for MyApp {
    type Target = SimulatorDisplay<Color>;
    type Msg = Msg;
    type Event = Event;
    type FocusKey = FocusKey;
    type AnyComponent<'a> = AnyComponent<'a>;
    type AnyPrimitive<'a> = AnyPrimitive<'a>;

    type Effect = Effect;

    fn initial_focus_key() -> Self::FocusKey {
        FocusKey::None
    }

    fn background_color() -> Color {
        Color::WHITE
    }

    fn new() -> Self {
        Self {
            text: Source::new("Hi".to_string()),
            font_style: Source::new(MonoTextStyle::new(&FONT_10X20, Color::RED)),
        }
    }

    fn update(&mut self, msg: Self::Msg) -> Change<Msg, FocusKey, Effect> {
        Change::none()
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
                // v.spacer(),
                v.centered(
                    Direction::Horizontal,
                    v.primitive(
                        Sizing::Intrinsic,
                        Text {
                            // content: self.text.signal_ref(),
                            content: SignalRef::constant(&"Hello!!!!"),
                            // font_style: self.font_style.signal(),
                            font_style: Signal::constant(
                                MonoTextStyleBuilder::new()
                                    .font(&embedded_graphics::mono_font::ascii::FONT_10X20)
                                    .text_color(Color::RED)
                                    .build(),
                            ),
                        },
                    ),
                ),
                // v.spacer(),
                // v.primitive(
                //     Sizing::Intrinsic,
                //     Text {
                //         content: self.text.signal_ref(),
                //         font_style: self.font_style.signal(),
                //     },
                // ),
                // v.component(
                //     Sizing::Fill,
                //     Button {
                //         text: SignalRef::constant(&"Say hi!"),
                //         font_style: self.font_style.signal(),
                //         size: Signal::constant(Size::new(128, 32)),
                //     },
                //     [],
                // ),
            ],
        )
    }
}

#[pollster::main]
async fn main() {
    let mut display =
        SimulatorDisplay::<Color>::new(embedded_graphics::prelude::Size::new(240, 280));

    let output_settings = OutputSettingsBuilder::new().build();

    let mut app = MyApp::new();
    let mut internal_state = InternalState::new(FocusKey::None);

    let mut window = Window::new("Hello World", &output_settings);
    // window.set_max_fps(60);

    // app::start(app, &mut display, || todo!(), after_render).await;
    // loop {
    display.clear(MyApp::background_color());
    app::render(&mut app, &mut internal_state, &mut display, true);
    window.show_static(&display);
    // }
}
