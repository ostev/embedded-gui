/// Trait for side-effects that can be run asynchronously after an update.
///
/// An effect returns an `Option<Msg>` so it can trigger further updates
/// (e.g. for async I/O or timers).
pub trait Effect {
    /// The message type this effect can produce.
    type Msg;

    /// The context type required to run this effect.
    type Context;

    /// Runs the effect asynchronously. Returns an optional message to
    /// be dispatched after completion.
    fn run(self, context: &mut Self::Context) -> impl Future<Output = Option<Self::Msg>>;
}
