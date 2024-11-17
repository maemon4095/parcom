use std::future::Future;

pub trait StreamSource: Clone {
    type Output;
    type Future: Future<Output = Option<Self::Output>>;
    fn next(&mut self) -> Self::Future;
}
