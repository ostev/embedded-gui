// use embedded_graphics::{
//     mono_font::{MonoTextStyle, iso_8859_13::FONT_10X20},
//     pixelcolor::{Rgb888, RgbColor},
// };
// use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
// use embedded_gui::{
//     Reactive,
//     app::{self, App},
//     component::button::Button,
//     interactive::FocusState,
//     layout::{Direction, Sizing},
//     primitive::text::Text,
//     signal::{Signal, SignalRef},
//     size::Size,
// };

// #[derive(Reactive)]
// struct MyApp {
//     text: Signal<String>,
//     font_style: Signal<MonoTextStyle<'static, Rgb888>>,
// }

// enum Msg {}

// #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
// enum FocusKey {
//     None,
// }

// impl App for MyApp {
//     type Target = SimulatorDisplay<Rgb888>;
//     type Msg = Msg;
//     type Event = ();
//     type FocusKey = FocusKey;

//     fn initial_focus_key() -> Self::FocusKey {
//         FocusKey::None
//     }

//     fn background_color() -> Rgb888 {
//         Rgb888::WHITE
//     }

//     fn new() -> Self {
//         Self {
//             text: Signal::new("Hi".to_string()),
//             font_style: Signal::new(MonoTextStyle::new(&FONT_10X20, Rgb888::RED)),
//         }
//     }

//     fn update(&mut self, msg: Self::Msg) -> Option<(FocusKey, FocusState)> {
//         None
//     }

//     fn view<'a>(
//         &'a self,
//         v: &'a embedded_gui::view::Factory<Self::FocusKey, Self::Event, Self::Msg>,
//     ) -> embedded_gui::view::View<'a, Self::Target, Self::FocusKey, Self::Event, Self::Msg> {
//         v.view(
//             Direction::Horizontal,
//             [
//                 v.spacer(),
//                 v.primitive(
//                     Sizing::Intrinsic,
//                     Text {
//                         content: self.text.to_ref(),
//                         font_style: self.font_style.to_ref(),
//                     },
//                 ),
//                 v.spacer(),
//                 v.primitive(
//                     Sizing::Intrinsic,
//                     Text {
//                         content: self.text.to_ref(),
//                         font_style: self.font_style.to_ref(),
//                     },
//                 ),
//                 v.component(
//                     Sizing::Fill,
//                     Button {
//                         text: SignalRef::owned("Say hi!"),
//                         font_style: self.font_style.to_ref(),
//                         size: SignalRef::owned(Size::new(128, 32)),
//                     },
//                     [],
//                 ),
//             ],
//         )
//     }
// }

// #[pollster::main]
// async fn main() {
//     let mut display =
//         SimulatorDisplay::<Rgb888>::new(embedded_graphics::prelude::Size::new(400, 400));

//     let output_settings = OutputSettingsBuilder::new().build();

//     let app = MyApp::new();

//     app::start(app, &mut display, || todo!(), after_render).await;
//     // Window::new("Hello World", &output_settings).update(&display);
// }
