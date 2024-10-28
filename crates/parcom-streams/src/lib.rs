pub mod slice;
pub mod str;
#[cfg(feature = "tokio-stream")]
pub mod tokio_stream;

pub struct Nodes<'me, T: ?Sized> {
    me: Option<&'me T>,
}

impl<'me, T: ?Sized> futures::Stream for Nodes<'me, T> {
    type Item = &'me T;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        std::task::Poll::Ready(self.me.take())
    }
}
