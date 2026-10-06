# `embedded-gui` (working title)

`embedded-gui` is a declarative GUI framework built on [`embedded-graphics`](https://docs.rs/embedded-graphics/latest/embedded_graphics/) for memory- and processor-constrained devices. It does not require `std`, and generally prioritises arena and stack allocations to limit heap fragmentation.

It allows you to write views using a declarative syntax, all within your Rust code:

```rust
v.view(
    Direction::Vertical,
    [
        v.sized_spacer(Signal::constant(Size::new(0, 20))),
        v.component(
            Sizing::Fill,
            Panel {
                info_1: ControlInfo {
                    progress: Signal::constant(0.35),
                    label: SignalRef::constant(&"Left"),
                },
                info_2: ControlInfo {
                    progress: Signal::constant(0.72),
                    label: SignalRef::constant(&"Right"),
                },
                color: Signal::constant(Color::On),
            },
            [],
        ),
    ],
)
```

Examples are located in the [`crates/embedded-gui/examples`](crates/embedded-gui/examples/) directory. Note that using this library or running any of the examples requires the nightly Rust toolchain, as `embedded-gui` makes heavy use of the new allocator API.

## 🚨 🚧 🚧 🚧 🚨 A word of caution

This library is not complete! The current implementation on `main` has some significant design choices that I would like to make differently. On the `rewrite` branch, I am currently in the process of rewriting the layout algorithm, completely changing the signals API, changing how messages are handled and writing better docs.
