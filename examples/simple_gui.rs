use embedded_graphics::{
    mono_font::{MonoTextStyle, iso_8859_13::FONT_10X20},
    pixelcolor::{BinaryColor, Rgb888},
    prelude::RgbColor,
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use embedded_gui::{
    Reactive,
    app::{self, App},
    component::button::Button,
    layout::{Direction, Sizing},
    primitive::text::Text,
    signal::{Signal, SignalRef},
    size::Size,
};

#[derive(Reactive)]
struct MyApp {
    text: Signal<String>,
    font_style: Signal<MonoTextStyle<'static, Rgb888>>,
}

enum Msg {}

impl App for MyApp {
    type Color = Rgb888;
    type Msg = Msg;

    fn default_focus_state() -> embedded_gui::interactive::FocusState {
        embedded_gui::interactive::FocusState::Unfocused
    }

    fn background_color() -> Self::Color {
        Rgb888::WHITE
    }

    fn init() -> Self {
        Self {
            text: Signal::new("Hi".to_string()),
            font_style: Signal::new(MonoTextStyle::new(&FONT_10X20, Rgb888::RED)),
        }
    }

    fn update(&mut self, msg: Self::Msg) {}

    fn view<'a>(
        &'a self,
        v: &'a embedded_gui::view::Factory,
    ) -> embedded_gui::view::View<'a, Self::Color> {
        v.view(
            Direction::Horizontal,
            [
                // v.spacer(),
                // v.primitive(
                //     Sizing::Intrinsic,
                //     Text {
                //         content: self.text.to_ref(),
                //         font_style: self.font_style.to_ref(),
                //     },
                // ),
                // v.spacer(),
                v.component(
                    Sizing::Intrinsic,
                    Button {
                        text: SignalRef::owned("Say hi!"),
                        font_style: self.font_style.to_ref(),
                        background_color: SignalRef::owned(Rgb888::BLUE),
                        size: SignalRef::owned(Size::new(128, 32)),
                    },
                    [v.primitive(
                        Sizing::Intrinsic,
                        Text {
                            content: self.text.to_ref(),
                            font_style: self.font_style.to_ref(),
                        },
                    )],
                ),
            ],
        )
    }
}

fn main() {
    let mut display =
        SimulatorDisplay::<Rgb888>::new(embedded_graphics::prelude::Size::new(400, 400));

    app::start(MyApp::init(), &mut display).unwrap();

    let output_settings = OutputSettingsBuilder::new().build();
    Window::new("Hello World", &output_settings).show_static(&display);
}
