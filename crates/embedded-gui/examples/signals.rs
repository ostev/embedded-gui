use embedded_gui::signal::Source;

fn main() {
    let mut counter = Source::new(4);
    println!("{:?}", counter.signal().map(|x| x + 1));

    counter.set_with(|value| value * 5);
    println!("{:?}", counter.signal().map(|x| x + 8))
}
