use std::future::Future;

pub trait SegmentStream {
    type SegmentRef;
    type Error;

    type Next<'a>: 'a + Future<Output = Result<Option<Self::SegmentRef>, Self::Error>>
    where
        Self: 'a;

    fn next<'a>(&'a mut self) -> Self::Next<'a>;
}
