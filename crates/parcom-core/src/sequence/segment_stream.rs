use std::future::Future;

pub trait SegmentStream {
    type Length;
    type Segment: ?Sized;

    type Next<'a>: 'a + Future<Output = Option<&'a Self::Segment>>
    where
        Self: 'a;

    fn next<'a>(&'a mut self, size_hint: Self::Length) -> Self::Next<'a>;
}
