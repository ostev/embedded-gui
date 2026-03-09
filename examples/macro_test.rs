use embedded_gui::element::Draw;
use embedded_gui_macros::draw_instruction;

struct Button {}

impl Draw for Button {
    fn draw(&self) {
        println!("Hello, world!");
    }
}

draw_instruction! {
    DrawInstruction {
        Button(Button)
    }
}

// macro_rules! new {
//     ($type_name:ident $properties:expr) => {
//         DrawInstruction::new_$type_name()
//     };
// }

fn main() {}
