use embedded_gui::{
    app::State,
    signal::{Signal, Source},
};

fn main() {
    let mut counter = Source::new(3);

    let mut list: Source<Vec<u32>> = Source::new(Vec::new());

    display_counter(counter.signal());

    counter.set_with(|x| x + 1);

    display_counter(counter.signal());

    counter.mark_resolved();

    display_counter(counter.signal());

    counter.set_with(|x| x * 2);
    display_counter(counter.signal());
}

fn display_counter(counter: Signal<'_, u32>) -> Signal<'_, ()> {
    let value = *counter?;
    println!("{}", value);
    let adjusted_value = value + 23;
    println!("{}", adjusted_value);

    // Signal::owned_constant(())
    Signal::finish()
}
