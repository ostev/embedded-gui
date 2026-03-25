use embedded_gui::arena::Arena;

fn main() {
    let mut arena = Arena::new();

    arena.alloc(1232);

    arena.set_capacity(20);
    let ptr = arena.alloc(123);
    println!("{}", arena[ptr]);

    for element in arena.iter() {
        println!("{}", element)
    }
}
