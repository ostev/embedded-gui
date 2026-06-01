pub trait Effect {
    type Msg;

    fn run(self) -> impl Future<Output = Self::Msg>;
}
