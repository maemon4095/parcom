use std::future::Future;

pub mod async_stream;
pub mod slice;
pub mod str;
pub mod util;

pub trait StreamSource: Clone {
    type Output;
    type Future: Future<Output = Option<Self::Output>>;
    fn recv(&mut self) -> Self::Future;
}
