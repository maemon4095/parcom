mod once_init;

pub use once_init::{InitializedSharedCell, OnceCell};

pub struct Nodes<'me, T: ?Sized> {
    me: Option<&'me T>,
}

impl<'me, T: ?Sized> Nodes<'me, T> {
    pub fn new(segment: &'me T) -> Self {
        Self { me: Some(segment) }
    }
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
