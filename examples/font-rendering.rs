use embedded_graphics::{
    Drawable, Pixel,
    pixelcolor::Rgb666,
    prelude::{Point, RgbColor, Size},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use glyphr::{AlignH, AlignV, Glyphr, RenderConfig, SdfConfig, TextAlign};

glyphr::generate_font! {
    name: INTER,
    path: "fonts/inter/Inter-Medium.ttf",
    size: 64,
    characters: "A-Za-z0-9 !$£%&",
    format: SDF {
        spread: 20.0,
        padding: 0,
    },
    // format: Bitmap {
    //     spread: 20.0,
    //     padding: 0
    // }
}

struct Display {
    display: SimulatorDisplay<Rgb666>,
}

impl Display {
    pub fn new() -> Self {
        Display {
            display: SimulatorDisplay::<Rgb666>::new(Size::new(256 * 4, 128 * 4)),
        }
    }
}

impl glyphr::RenderTarget for Display {
    fn write_pixel(&mut self, x: u32, y: u32, color: u32) -> bool {
        let intensity = color.to_be_bytes()[0];
        Pixel(
            Point::new(x as i32, y as i32),
            Rgb666::new(intensity, intensity, intensity),
        )
        .draw(&mut self.display)
        .unwrap();

        true
    }

    fn dimensions(&self) -> (u32, u32) {
        (256 * 4, 128 * 4)
    }
}

fn main() {
    let mut display = Display::new();

    let config = RenderConfig {
        color: 0xffffff,
        sdf: SdfConfig {
            size: 64,
            mid_value: 0.5,
            smoothing: 0.5,
        },
    };

    let renderer = Glyphr::with_config(config);

    renderer
        .render(
            &mut display,
            "hello",
            INTER,
            20,
            30,
            TextAlign {
                horizontal: AlignH::Left,
                vertical: AlignV::Top,
            },
        )
        .unwrap();

    let output_settings = OutputSettingsBuilder::new().build();
    Window::new("Hello World", &output_settings).show_static(&display.display);
}
