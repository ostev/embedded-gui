pub trait Effect<Msg> {
    fn run(self) -> impl Future<Output = Msg>;
}
