use embedded_gui_macros::draw_instruction;

#[draw_instruction]
enum CustomDrawInstruction {
    Button { text: String, color: u32 },
    Panel(u32, u16),
}

fn main() {}
