use embedded_gui::arena::Arena;

fn main() {
    let mut arena = Arena::new();

    arena.realloc(20);
    arena.push(123);

    for element in arena.iter() {
        println!("{}", element)
    }
}
