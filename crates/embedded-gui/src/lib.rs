//! A reactive GUI framework for embedded systems.
//!
//! `embedded-gui` is built on top of [`embedded-graphics`] and follows an
//! Elm-inspired Model-View-Update architecture. It is designed to run in
//! `#![no_std]` environments such as microcontrollers.
//!
//! # Important types
//!
//! - **App** – the top-level trait that defines your application's state,
//!   update logic, and view hierarchy.
//! - **Component** – a reusable widget that produces a `View` subtree.
//! - **Primitive** – a widget that is drawn directly to the screen
//! - **Source** – a reactive wrapper that tracks whether a value has changed,
//!   enabling incremental rendering.
//! - **Factory** – a builder that constructs the view tree.
//!
//! # Cargo Features
//!
//! - `clipping` – enables pixel-level bounds checking when drawing
//!   (disabled by default for performance).

#![no_std]
#![feature(allocator_api)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

extern crate alloc;
extern crate self as embedded_gui;

pub mod app;
pub mod component;
pub mod draw;
pub mod effect;
pub mod event;
pub mod interactive;
pub mod layout;
pub mod position;
pub mod primitive;
pub mod signal;
pub mod size;
pub mod view;
