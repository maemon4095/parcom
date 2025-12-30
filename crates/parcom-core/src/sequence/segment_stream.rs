use std::future::Future;

use crate::SequenceSegment;

pub trait SegmentStream {
    type Segment: ?Sized + SequenceSegment;

    type Next<'a>: 'a + Future<Output = Option<&'a Self::Segment>>
    where
        Self: 'a;

    fn next<'a>(
        &'a mut self,
        size_hint: <Self::Segment as SequenceSegment>::Length,
    ) -> Self::Next<'a>;
}
