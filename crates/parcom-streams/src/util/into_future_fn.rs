use std::future::IntoFuture;

pub fn into_future_fn<Fut, F>(f: F) -> FnOnceFuture<Fut, F>
where
    Fut: IntoFuture,
    F: FnOnce() -> Fut,
{
    FnOnceFuture { f }
}

pub struct FnOnceFuture<Fut, F>
where
    Fut: IntoFuture,
    F: FnOnce() -> Fut,
{
    f: F,
}

impl<Fut, F> IntoFuture for FnOnceFuture<Fut, F>
where
    Fut: IntoFuture,
    F: FnOnce() -> Fut,
{
    type Output = Fut::Output;
    type IntoFuture = Fut::IntoFuture;

    fn into_future(self) -> Self::IntoFuture {
        (self.f)().into_future()
    }
}
