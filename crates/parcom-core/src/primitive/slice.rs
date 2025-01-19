use super::{Anchor, Nodes};
use crate::{IntoMeasured, MeasuredStream, Meter, Metrics, RewindStream, Stream, StreamSegment};

impl<'a, T> Stream for &'a [T] {
    type Segment = [T];
    type SegmentIter = Nodes<'a, [T]>;
    type Advance = std::future::Ready<Self>;

    fn segments(&self) -> Self::SegmentIter {
        Nodes { me: Some(self) }
    }

    fn advance(self, count: usize) -> Self::Advance {
        std::future::ready(&self[count..])
    }
}

impl<T> RewindStream for &[T] {
    type Anchor = Anchor<Self>;
    type Rewind = std::future::Ready<Self>;

    fn anchor(&self) -> Self::Anchor {
        Anchor { me: self }
    }

    fn rewind(self, anchor: Self::Anchor) -> Self::Rewind {
        let ptr = anchor.me.as_ptr();
        let len = anchor.me.len();
        let offset = unsafe { self.as_ptr().offset_from(ptr) };
        if !offset.is_negative() && (offset as usize) <= len {
            std::future::ready(anchor.me)
        } else {
            panic!("the anchor is not an anchor of this stream.")
        }
    }
}

impl<T> StreamSegment for [T] {
    type Delta = usize;

    fn slice(&self, delta: Self::Delta) -> &Self {
        &self[delta..]
    }
}

impl<'me, T> IntoMeasured for &'me [T] {
    type Measured<M: Metrics<Self::Segment>> = Measured<'me, T, M>;

    fn into_measured_with<M: Metrics<Self::Segment>>(self, meter: M::Meter) -> Self::Measured<M> {
        Measured { meter, base: self }
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

impl<'me, T, M: Metrics<[T]>> Stream for Measured<'me, T, M> {
    type Segment = [T];
    type SegmentIter = Nodes<'me, [T]>;
    type Advance = std::future::Ready<Self>;

    fn segments(&self) -> Self::SegmentIter {
        self.base.segments()
    }

    fn advance(mut self, delta: usize) -> Self::Advance {
        let segment = self.base;
        let end = segment.len().min(delta);
        self.meter = self.meter.advance(&segment[..end]);
        self.base = self.base.advance(delta).into_inner();
        std::future::ready(self)
    }
}

impl<'me, T, M> RewindStream for Measured<'me, T, M>
where
    M: Metrics<[T]>,
    M::Meter: Clone,
{
    type Anchor = Anchor<Self>;
    type Rewind = std::future::Ready<Self>;

    fn anchor(&self) -> Self::Anchor {
        Anchor { me: self.clone() }
    }

    fn rewind(self, anchor: Self::Anchor) -> Self::Rewind {
        std::future::ready(anchor.me)
    }
}

impl<'me, T, M: Metrics<[T]>> MeasuredStream for Measured<'me, T, M> {
    type Metrics = M;

    fn metrics(&self) -> Self::Metrics {
        self.meter.metrics()
    }
}
