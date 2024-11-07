use std::future::Future;

pub mod slice;
pub mod str;
pub mod streams;
pub mod util;

pub trait StreamSource: Clone {
    type Output;
    type Future: Future<Output = Option<Self::Output>>;
    fn recv(&mut self) -> Self::Future;
}
