use embedded_gui::arena::Arena;

static mut ARENA_1: Arena<i16> = Arena::new();

fn main() {
    let mut arena = Arena::new();
    let mut arena2 = Arena::new();

    unsafe { ARENA_1.push(14) };

    arena.push(1232);

    arena.set_capacity(20);
    let ptr = arena.push(123);
    println!("{}", arena2[ptr]);

    for element in arena.iter() {
        println!("{}", element)
    }
}
