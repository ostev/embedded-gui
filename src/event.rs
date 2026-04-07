pub type Subscription<Msg> = dyn Future<Output = Msg>;
