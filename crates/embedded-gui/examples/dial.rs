#![feature(allocator_api)]

extern crate alloc;

use core::{f32, future::Future, future::ready};

use embedded_graphics::{
    geometry::{Angle, Point},
    mono_font::{MonoFont, MonoTextStyle, ascii},
    pixelcolor::BinaryColor,
    primitives::{Circle, PrimitiveStyle, Sector, StyledDrawable},
};
use embedded_gui::{
    app::{self, App, State},
    component::{Component, any_component, group::Group},
    effect::Effect,
    layout::{Direction, IntrinsicSize, Sizing},
    primitive::{Primitive, any_primitive, spacer::Spacer, text::Text},
    signal::{Reactive, Signal, SignalRef},
    size::Size,
};

type Color = BinaryColor;
type Display = embedded_graphics_simulator::SimulatorDisplay<Color>;

/// Circular progress indicator used to display a normalized parameter value.
///
/// The dial is rendered as a filled sector with an outline ring.
#[derive(Reactive)]
pub struct Dial {
    /// Progress should be between 0.0 and 1.0
    pub progress: Signal<f32>,
    pub color: Signal<Color>,
}

/// Radius of the dial, including the outline.
const RADIUS: u16 = 20;
// The thickness of the outline
const OUTLINE_THICKNESS: u32 = 3;

impl IntrinsicSize for Dial {
    fn intrinsic_size(&self) -> Size {
        // The bounding box of the circle, plus a bit of padding
        Size::new(RADIUS * 2 + 2, RADIUS * 2 + 2)
    }
}

impl Primitive<Display> for Dial {
    fn draw(
        &self,
        target: &mut embedded_gui::draw::LocalTarget<Display>,
    ) -> Result<(), <Display as embedded_graphics::prelude::DrawTarget>::Error> {
        let mut outline_style = PrimitiveStyle::with_stroke(*self.color, OUTLINE_THICKNESS);
        outline_style.stroke_alignment = embedded_graphics::primitives::StrokeAlignment::Inside;

        let fill_style = PrimitiveStyle::with_fill(*self.color);

        let outline = Circle::new(Point::new(0, 0), RADIUS as u32 * 2);

        // Ensure the sector is always visible, even for values very close to zero.
        // The sector will start to become triangular at lower angles.
        const MIN_PROGRESS: f32 = 0.03;

        let angle = self.progress.clamp(MIN_PROGRESS, 1.0) * 2.0 * f32::consts::PI;

        // Draw the fill sector
        Sector::from_circle(
            outline,
            Angle::from_radians(-(f32::consts::PI / 2.0)), // Draw from the top of the circle
            Angle::from_radians(angle),
        )
        .draw_styled(&fill_style, target)?;

        // Draw the outline ring
        outline.draw_styled(&outline_style, target)?;

        Ok(())
    }
}

/// State for a single dial control
#[derive(Reactive, Clone)]
pub struct ControlInfo {
    /// Progress should be between 0.0 and 1.0
    pub progress: Signal<f32>,
    pub label: SignalRef<'static, &'static str>,
}

/// Displays a single parameter as a dial with a text label beneath it.
#[derive(Reactive)]
pub struct Control {
    pub color: Signal<Color>,

    pub info: ControlInfo,
}

const FONT: MonoFont = ascii::FONT_10X20;

impl IntrinsicSize for Control {
    fn intrinsic_size(&self) -> Size {
        unimplemented!("Dial controls only support fill sizing!")
    }
}

impl<'a> Component<'a, Display, (), (), (), AnyComponent<'a>, AnyPrimitive<'a>> for Control {
    fn view(
        &self,
        v: &'a embedded_gui::view::Factory<(), (), ()>,
        _: embedded_gui::view::Children<
            'a,
            Display,
            (),
            (),
            (),
            AnyComponent<'a>,
            AnyPrimitive<'a>,
        >,
    ) -> embedded_gui::view::View<'a, Display, (), (), (), AnyComponent<'a>, AnyPrimitive<'a>> {
        v.view(
            Direction::Vertical,
            [
                v.centered(
                    Direction::Horizontal,
                    v.primitive(
                        Sizing::Intrinsic,
                        Dial {
                            progress: self.info.progress,
                            color: self.color.clone(),
                        },
                    ),
                ),
                // Label centered below the dial
                v.centered(
                    Direction::Horizontal,
                    v.primitive(
                        Sizing::Intrinsic,
                        Text {
                            content: self.info.label.clone(),
                            font_style: Signal::constant(MonoTextStyle::new(&FONT, Color::On)),
                        },
                    ),
                ),
            ],
        )
    }
}

/// Displays two dial controls side-by-side with a fixed gap between them.
#[derive(Reactive)]
pub struct Panel {
    pub info_1: ControlInfo,
    pub info_2: ControlInfo,

    pub color: Signal<Color>,
}

const MARGIN: u16 = 20;

impl IntrinsicSize for Panel {
    fn intrinsic_size(&self) -> Size {
        unimplemented!("Dial panels only support fill layout!")
    }
}

impl<'a> Component<'a, Display, (), (), (), AnyComponent<'a>, AnyPrimitive<'a>> for Panel {
    fn view(
        &self,
        v: &'a embedded_gui::view::Factory<(), (), ()>,
        _: embedded_gui::view::Children<
            'a,
            Display,
            (),
            (),
            (),
            AnyComponent<'a>,
            AnyPrimitive<'a>,
        >,
    ) -> embedded_gui::view::View<'a, Display, (), (), (), AnyComponent<'a>, AnyPrimitive<'a>> {
        v.view(
            Direction::Horizontal,
            [
                v.component(
                    Sizing::Fill,
                    Control {
                        info: self.info_1.clone(),
                        color: self.color.clone(),
                    },
                    [],
                ),
                v.primitive(
                    Sizing::Intrinsic,
                    Spacer {
                        size: Signal::constant(Size::new(MARGIN, 0)),
                    },
                ),
                v.component(
                    Sizing::Fill,
                    Control {
                        info: self.info_2.clone(),
                        color: self.color.clone(),
                    },
                    [],
                ),
            ],
        )
    }
}

#[derive(Reactive)]
#[any_component(target = Display, event = (), msg = (), focus_key = (), any_primitive = AnyPrimitive<'a>)]
enum AnyComponent<'a> {
    Control(Control),
    Panel(Panel),
    Group(Group),
}

#[derive(Reactive)]
#[any_primitive(target = Display)]
enum AnyPrimitive<'a> {
    Dial(Dial),
    Spacer(Spacer),
    Text(Text<'a, Color, &'static str>),
}

struct NoEffect;

impl Effect for NoEffect {
    type Msg = ();
    type Context = ();

    fn run(self, _: &mut Self::Context) -> impl Future<Output = Option<Self::Msg>> {
        ready(None)
    }
}

#[derive(Reactive, State)]
struct DialApp;

impl App for DialApp {
    type Target = Display;
    type Msg = ();
    type Event = ();
    type FocusKey = ();
    type AnyComponent<'a> = AnyComponent<'a>;
    type AnyPrimitive<'a> = AnyPrimitive<'a>;
    type Effect = NoEffect;

    fn new() -> Self {
        Self
    }

    fn initial_focus_key() -> Self::FocusKey {}

    fn background_color() -> Color {
        Color::Off
    }

    fn update(&mut self, _: Self::Msg) -> app::Change<Self::Msg, Self::FocusKey, Self::Effect> {
        app::Change::new()
    }

    fn view<'a>(
        &'a self,
        v: &'a embedded_gui::view::Factory<Self::Event, Self::Msg, Self::FocusKey>,
    ) -> embedded_gui::view::View<'a, Display, (), (), (), AnyComponent<'a>, AnyPrimitive<'a>> {
        v.view(
            Direction::Vertical,
            [
                v.sized_spacer(Signal::constant(Size::new(0, 20))),
                v.component(
                    Sizing::Fill,
                    Panel {
                        info_1: ControlInfo {
                            progress: Signal::constant(0.35),
                            label: SignalRef::constant(&"DIAL 1"),
                        },
                        info_2: ControlInfo {
                            progress: Signal::constant(0.72),
                            label: SignalRef::constant(&"DIAL 2"),
                        },
                        color: Signal::constant(Color::On),
                    },
                    [],
                ),
            ],
        )
    }

    fn global_event_handler(&self, _: Self::Event) -> Option<Self::Msg> {
        None
    }
}

fn main() {
    use embedded_graphics::prelude::Size as DisplaySize;
    use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorEvent, Window};

    let mut display = Display::new(DisplaySize::new(160, 100));
    let output_settings = OutputSettingsBuilder::new().scale(4).build();
    let mut window = Window::new("embedded-gui dial", &output_settings);
    let mut app = DialApp::new();
    let mut internal_state = app::InternalState::new(DialApp::initial_focus_key());
    let mut effect_context = ();

    pollster::block_on(app::dispatch_msg(
        &mut app,
        &mut effect_context,
        &mut internal_state,
        (),
    ));
    app::render(&mut app, &mut internal_state, &mut display, true).unwrap();
    window.update(&display);

    'running: loop {
        for event in window.events() {
            if let SimulatorEvent::Quit = event {
                break 'running;
            }
        }
    }
}
