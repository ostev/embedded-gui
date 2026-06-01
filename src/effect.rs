pub trait Effect<Msg> {
    fn run(&mut self) -> impl Future<Output = Msg>;
}
