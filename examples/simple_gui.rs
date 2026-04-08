use embedded_graphics::{
    mono_font::{MonoTextStyle, iso_8859_13::FONT_10X20},
    pixelcolor::{BinaryColor, Rgb888},
    prelude::RgbColor,
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use embedded_gui::{
    Reactive,
    app::{self, App},
    layout::{Direction, Layout, Sizing},
    primitive::text::Text,
    signal::{Signal, SignalRef},
};

#[derive(Reactive)]
struct MyApp {
    text: Signal<String>,
}

enum Msg {}

impl App for MyApp {
    type FocusState = ();

    type Color = Rgb888;
    type Msg = Msg;

    fn default_focus_state() -> Self::FocusState {
        ()
    }

    fn background_color() -> Self::Color {
        Rgb888::WHITE
    }

    fn init() -> Self {
        let mut text = Signal::new("Hello".to_string());
        text.set("Hi!!".to_string());
        Self { text }
    }

    fn update(&mut self, msg: Self::Msg) {}

    fn view<'a>(
        &'a self,
        v: &'a embedded_gui::view::Factory,
    ) -> embedded_gui::view::View<'a, Self::FocusState, Self::Color> {
        v.view(
            Layout::new(Sizing::Fill, Direction::Horizontal),
            [
                v.primitive(
                    Text {
                        content: self.text.to_ref(),
                        font_style: SignalRef::owned(MonoTextStyle::new(&FONT_10X20, Rgb888::RED)),
                    },
                    Layout::new(Sizing::Fill, Direction::Horizontal),
                ),
                v.primitive(
                    Text {
                        content: self.text.to_ref(),
                        font_style: SignalRef::owned(MonoTextStyle::new(&FONT_10X20, Rgb888::RED)),
                    },
                    Layout::new(Sizing::Intrinsic, Direction::Horizontal),
                ),
            ],
        )
    }
}

fn main() {
    let mut display =
        SimulatorDisplay::<Rgb888>::new(embedded_graphics::prelude::Size::new(400, 64));

    app::start(MyApp::init(), &mut display).unwrap();

    let output_settings = OutputSettingsBuilder::new().build();
    Window::new("Hello World", &output_settings).show_static(&display);
}
