pub trait Effect {
    type Msg;
    type Context;

    fn run(self, context: &mut Self::Context) -> impl Future<Output = Self::Msg>;
}
