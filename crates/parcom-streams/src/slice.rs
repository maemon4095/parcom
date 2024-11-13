use super::util::Nodes;
use parcom_core::{IntoMeasured, MeasuredStream, Meter, Metrics, ParcomStream, RewindStream};

#[derive(Debug)]
pub struct SliceStream<'me, T> {
    slice: &'me [T],
}

impl<'me, T> SliceStream<'me, T> {
    pub fn new(slice: &'me [T]) -> Self {
        Self { slice }
    }
}

impl<'me, T> Clone for SliceStream<'me, T> {
    fn clone(&self) -> Self {
        Self { slice: self.slice }
    }
}

impl<'me, T> ParcomStream for SliceStream<'me, T> {
    type Segment = [T];
    type SegmentStream = Nodes<'me, [T]>;
    type Advance = std::future::Ready<Self>;

    fn segments(&self) -> Self::SegmentStream {
        Nodes::new(&self.slice)
    }
    fn advance(mut self, count: usize) -> Self::Advance {
        self.slice = &self.slice[count..];
        std::future::ready(self)
    }
}

impl<'me, T> RewindStream for SliceStream<'me, T> {
    type Anchor = Anchor<'me, T>;

    fn anchor(&self) -> Self::Anchor {
        Anchor {
            stream: self.clone(),
        }
    }

    fn rewind(self, anchor: Self::Anchor) -> Self {
        anchor.stream
    }
}

pub struct Anchor<'me, T> {
    stream: SliceStream<'me, T>,
}

impl<'me, T, M> IntoMeasured<M> for SliceStream<'me, T>
where
    M: Metrics<[T]>,
{
    type Measured = Measured<'me, T, M>;

    fn into_measured_with(self, meter: M::Meter) -> Self::Measured {
        Measured { meter, base: self }
    }
}

#[derive(Debug)]
pub struct Measured<'me, T, M>
where
    M: Metrics<[T]>,
{
    meter: M::Meter,
    base: SliceStream<'me, T>,
}

impl<'me, T, M> Clone for Measured<'me, T, M>
where
    M: Metrics<[T]>,
    M::Meter: Clone,
{
    fn clone(&self) -> Self {
        Self {
            meter: self.meter.clone(),
            base: self.base.clone(),
        }
    }
}

impl<'me, T, M> ParcomStream for Measured<'me, T, M>
where
    M: Metrics<[T]>,
{
    type Segment = <SliceStream<'me, T> as ParcomStream>::Segment;
    type SegmentStream = Nodes<'me, [T]>;
    type Advance = std::future::Ready<Self>;

    fn segments(&self) -> Self::SegmentStream {
        Nodes::new(&self.base.slice)
    }

    fn advance(mut self, count: usize) -> Self::Advance {
        let segment = self.base.slice;
        self.meter = self.meter.advance(&segment[..count]);
        self.base = self.base.advance(count).into_inner();
        std::future::ready(self)
    }
}

impl<'me, T, M> RewindStream for Measured<'me, T, M>
where
    M: Metrics<[T]>,
    M::Meter: Clone,
{
    type Anchor = MeasuredAnchor<'me, T, M>;

    fn anchor(&self) -> Self::Anchor {
        MeasuredAnchor {
            stream: self.clone(),
        }
    }

    fn rewind(self, anchor: Self::Anchor) -> Self {
        anchor.stream
    }
}

pub struct MeasuredAnchor<'me, T, M>
where
    M: Metrics<[T]>,
{
    stream: Measured<'me, T, M>,
}

impl<'me, T, M> MeasuredStream for Measured<'me, T, M>
where
    M: Metrics<[T]>,
{
    type Metrics = M;

    fn metrics(&self) -> Self::Metrics {
        self.meter.metrics()
    }
}
