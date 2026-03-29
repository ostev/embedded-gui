use embedded_gui::{component::Component, primitive::Primitive, view::Widget};

struct Button {
    background_color: u32,
    width: u32,
    height: u32,
    depth: u32,
}

type Index = u16;

enum DrawInstruction {
    Panel,
    Button(Index),
}

fn draw_button(properties: &ButtonProperties) {}

impl DrawInstruction {
    pub fn draw(&self, properties: Properties) {
        match self {
            Self::Button(index) => {
                draw_button(&properties.button[*index as usize]);
            }
            _ => todo!(),
        }
    }
}

struct ButtonProperties {
    color: u32,
    text: String,
}

struct Properties {
    button: Vec<ButtonProperties>,
}

fn main() {
    println!("{}", core::mem::size_of::<DrawInstruction>());
}
