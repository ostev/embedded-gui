#![no_std]
#![feature(arbitrary_self_types)]

use embedded_gui_macros::{Reactive, any_component};

extern crate alloc;
extern crate self as embedded_gui;

pub mod app;
pub mod background;
pub mod component;
pub mod draw;
pub mod event;
pub mod interactive;
pub mod layout;
pub mod position;
pub mod primitive;
pub mod signal;
pub mod size;
pub mod view;

// macro_rules! any_component {
//     (components $name:ident {$($variant:ident => $component:ty),+}) => {
//         impl<'a> $crate::signal::Reactive for $name<'a> {
//             fn has_changed(&self) -> bool {
//                 match self {
//                     $($variant(component) => component.has_changed()),+
//                 }
//             }
//         }

//         impl<'a> $crate::component::Component for $name<'a> {
//             fn has_changed(&self) -> bool {
//                 match self {
//                     $($variant(component) => component.has_changed()),+
//                 }
//             }
//         }
//     };
// }
