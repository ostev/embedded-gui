#![feature(trace_macros)]

use embedded_gui::{arena::Arena, draw, primitive::Primitive, signal::Signal};
use embedded_gui_macros::{Reactive, primitives};

#[derive(Reactive)]
struct Button {
    text: Signal<String>,
    width: Signal<usize>,
    height: Signal<usize>,
}

impl Primitive for Button {
    fn draw(&self, target: impl draw::Target) {
        println!("Hello, world!");
    }
}

primitives! {
    Primitives {
        Button(Button)
    }
}

fn main() {
    let mut arena = Arena::new();
    let button = Primitives::new_Button(
        &mut arena,
        Button {
            text: todo!(),
            width: todo!(),
            height: todo!(),
        },
    );
}
