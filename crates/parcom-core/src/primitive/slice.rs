use super::{Anchor, Nodes};
use crate::{
    measured::{IntoMeasured, Meter, Metrics},
    MeasuredStream, Never, PeekableStream, RewindStream, Stream, StreamSegment,
};

impl<'a, T: 'static> Stream for &'a [T] {
    type Segment = [T];
    type Error = Never;
    type SegmentIter<'b>
        = Nodes<'a, [T]>
    where
        Self: 'b;
    type Advance = std::future::Ready<Result<Self, Never>>;

    fn segments(&mut self) -> Self::SegmentIter<'_> {
        Nodes { me: Some(self) }
    }

    fn advance(self, count: usize) -> Self::Advance {
        std::future::ready(Ok(&self[count..]))
    }
}

impl<T: 'static> RewindStream for &[T] {
    type Anchor = Anchor<Self>;
    type Rewind = std::future::Ready<Result<Self, Never>>;

    fn anchor(&self) -> Self::Anchor {
        Anchor { me: self }
    }

    fn rewind(self, anchor: Self::Anchor) -> Self::Rewind {
        let ptr = anchor.me.as_ptr();
        let len = anchor.me.len();
        let offset = unsafe { self.as_ptr().offset_from(ptr) };
        if !offset.is_negative() && (offset as usize) <= len {
            std::future::ready(Ok(anchor.me))
        } else {
            panic!("the anchor is not an anchor of this stream.")
        }
    }
}

impl<T: 'static> StreamSegment for [T] {
    type Length = usize;

    fn len(&self) -> Self::Length {
        <[T]>::len(self)
    }

    fn split_at(&self, mid: Self::Length) -> (&Self, &Self) {
        <[T]>::split_at(self, mid)
    }
}

impl<'me, T: 'static> IntoMeasured for &'me [T] {
    type Measured<M: Metrics<Self::Segment>> = Measured<'me, T, M>;

    fn into_measured_with<M: Metrics<Self::Segment>>(self, meter: M::Meter) -> Self::Measured<M> {
        Measured { meter, base: self }
    }
}

impl<T: 'static> PeekableStream for &[T] {
    type Peek<'a>
        = Self
    where
        Self: 'a;

    fn peek(&mut self) -> Self::Peek<'_> {
        self
    }
}

#[derive(Debug)]
pub struct Measured<'me, T, M: Metrics<[T]>> {
    meter: M::Meter,
    base: &'me [T],
}

impl<'me, T, M> Clone for Measured<'me, T, M>
where
    M::Meter: Clone,
    M: Metrics<[T]>,
{
    fn clone(&self) -> Self {
        Self {
            meter: self.meter.clone(),
            base: self.base,
        }
    }
}

impl<'me, T: 'static, M: Metrics<[T]>> Stream for Measured<'me, T, M> {
    type Segment = [T];
    type Error = Never;
    type SegmentIter<'a>
        = Nodes<'me, [T]>
    where
        Self: 'a;
    type Advance = std::future::Ready<Result<Self, Never>>;

    fn segments(&mut self) -> Self::SegmentIter<'_> {
        self.base.segments()
    }

    fn advance(mut self, delta: usize) -> Self::Advance {
        let segment = self.base;
        let end = segment.len().min(delta);
        self.meter = self.meter.advance(&segment[..end]);
        self.base = self.base.advance(delta).into_inner().unwrap();
        std::future::ready(Ok(self))
    }
}

impl<'me, T: 'static, M> RewindStream for Measured<'me, T, M>
where
    M: Metrics<[T]>,
    M::Meter: Clone,
{
    type Anchor = Anchor<Self>;
    type Rewind = std::future::Ready<Result<Self, Never>>;

    fn anchor(&self) -> Self::Anchor {
        Anchor { me: self.clone() }
    }

    fn rewind(self, anchor: Self::Anchor) -> Self::Rewind {
        std::future::ready(Ok(anchor.me))
    }
}

impl<'me, T: 'static, M: Metrics<[T]>> MeasuredStream for Measured<'me, T, M> {
    type Metrics = M;

    fn metrics(&self) -> Self::Metrics {
        self.meter.metrics()
    }
}
