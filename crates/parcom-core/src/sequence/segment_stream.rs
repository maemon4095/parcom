use std::future::Future;

pub trait SegmentStream {
    type SegmentRef;

    type Next<'a>: 'a + Future<Output = Option<Self::SegmentRef>>
    where
        Self: 'a;

    fn next<'a>(&'a mut self) -> Self::Next<'a>;
}
