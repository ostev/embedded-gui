#![feature(trace_macros)]

use bumpalo::Bump;
use embedded_gui::{component::Component, draw, primitive::Primitive, signal::Signal};
use embedded_gui_macros::{Reactive, widgets};

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

#[derive(Reactive)]
struct Container {}

impl<'a> Component<'a, Primitives<'a>, Components<'a>> for Container {
    fn view(&self, bump: &Bump, children: embedded_gui::view::View<'a, Primitives, Components>) {
        todo!()
    }
}

widgets! {
    pub Primitives {
        Button(Button)
    }

    pub Components {
        Container(Container)
    }
}

// macro_rules! view {
//     ($bump:expr, $($constructor:ty $props:expr => [$($child:expr,)*]),*) => {
//         $(
//             if ::embedded_gui::signal::Reactive::has_changed($props) {

//             } else {

//             }
//         )*
//     };
// }

fn main() {
    let bump = Bump::new();
    // let button = button!(
    //     &bump,
    //     Button {
    //         text: Signal::new("Hello".to_string()),
    //         width: Signal::new(40),
    //         height: Signal::new(40)
    //     }
    // );
    // ::embedded_gui::view::Widget::Primitive<'a, Primitives, ()>(&button);
}
