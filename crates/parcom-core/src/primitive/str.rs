use super::{Anchor, BytesDelta, Nodes};
use crate::{
    measured::{IntoMeasured, Meter, Metrics},
    MeasuredStream, Never, PeekableStream, RewindStream, Stream, StreamSegment,
};

impl<'a> Stream for &'a str {
    type Segment = str;
    type Error = Never;
    type SegmentIter<'b>
        = Nodes<'a, str>
    where
        Self: 'b;
    type Advance = std::future::Ready<Result<Self, Never>>;

    fn segments(&mut self) -> Self::SegmentIter<'_> {
        Nodes { me: Some(self) }
    }

    fn advance(self, delta: BytesDelta) -> Self::Advance {
        let delta = delta.to_bytes();
        let rest = self.get(delta..).unwrap_or("");
        std::future::ready(Ok(rest))
    }
}

impl RewindStream for &str {
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

impl<'me> IntoMeasured for &'me str {
    type Measured<M: Metrics<Self::Segment>> = Measured<'me, M>;

    fn into_measured_with<M: Metrics<Self::Segment>>(self, meter: M::Meter) -> Self::Measured<M> {
        Measured { meter, base: self }
    }
}

impl PeekableStream for &str {
    type Peek<'a>
        = Self
    where
        Self: 'a;

    fn peek(&mut self) -> Self::Peek<'_> {
        self
    }
}

#[derive(Debug)]
pub struct Measured<'me, M: Metrics<str>> {
    meter: M::Meter,
    base: &'me str,
}

impl<'me, M> Clone for Measured<'me, M>
where
    M::Meter: Clone,
    M: Metrics<str>,
{
    fn clone(&self) -> Self {
        Self {
            meter: self.meter.clone(),
            base: self.base,
        }
    }
}

impl<'me, M: Metrics<str>> Stream for Measured<'me, M> {
    type Segment = str;
    type Error = Never;
    type SegmentIter<'b>
        = Nodes<'me, str>
    where
        Self: 'b;
    type Advance = std::future::Ready<Result<Self, Never>>;

    fn segments(&mut self) -> Self::SegmentIter<'_> {
        self.base.segments()
    }

    fn advance(mut self, delta: BytesDelta) -> Self::Advance {
        let segment = self.base;
        let end = segment.len().min(delta.to_bytes());
        self.meter = self.meter.advance(&segment[..end]);
        self.base = self.base.advance(delta).into_inner().unwrap();
        std::future::ready(Ok(self))
    }
}

impl<'me, M> RewindStream for Measured<'me, M>
where
    M: Metrics<str>,
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

impl<'me, M: Metrics<str>> MeasuredStream for Measured<'me, M> {
    type Metrics = M;

    fn metrics(&self) -> Self::Metrics {
        self.meter.metrics()
    }
}

impl StreamSegment for str {
    type Length = BytesDelta;

    fn len(&self) -> Self::Length {
        BytesDelta::from_str(self)
    }

    fn split_at(&self, mid: Self::Length) -> (&Self, &Self) {
        str::split_at(&self, mid.0)
    }
}
