use parcom_core::{
    primitive::{BytesDelta, Nodes},
    IntoMeasured, MeasuredStream, Meter, Metrics, RewindStream, Stream,
};

#[derive(Debug, Clone)]
pub struct StrStream<'me> {
    str: &'me str,
}

impl<'me> StrStream<'me> {
    pub fn new(str: &'me str) -> Self {
        Self { str }
    }
}

impl<'me> Stream for StrStream<'me> {
    type Segment = str;
    type SegmentIter = Nodes<'me, str>;
    type Advance = std::future::Ready<Self>;

    fn segments(&self) -> Self::SegmentIter {
        self.str.segments()
    }

    fn advance(mut self, delta: BytesDelta) -> Self::Advance {
        let delta: usize = delta.into();
        let end = self.str.len().min(delta);
        self.str = &self.str[..end];
        std::future::ready(self)
    }
}
impl<'me> RewindStream for StrStream<'me> {
    type Anchor = Anchor<'me>;
    type Rewind = std::future::Ready<Self>;

    fn anchor(&self) -> Self::Anchor {
        Anchor {
            stream: self.clone(),
        }
    }

    fn rewind(self, anchor: Self::Anchor) -> Self::Rewind {
        std::future::ready(anchor.stream)
    }
}

pub struct Anchor<'me> {
    stream: StrStream<'me>,
}

impl<'me> IntoMeasured for StrStream<'me> {
    type Measured<M: Metrics<Self::Segment>> = Measured<'me, M>;

    fn into_measured_with<M: Metrics<Self::Segment>>(self, meter: M::Meter) -> Self::Measured<M> {
        Measured { meter, base: self }
    }
}

#[derive(Debug)]
pub struct Measured<'me, M: Metrics<str>> {
    meter: M::Meter,
    base: StrStream<'me>,
}

impl<'me, M> Clone for Measured<'me, M>
where
    M::Meter: Clone,
    M: Metrics<str>,
{
    fn clone(&self) -> Self {
        Self {
            meter: self.meter.clone(),
            base: self.base.clone(),
        }
    }
}

impl<'me, M: Metrics<str>> Stream for Measured<'me, M> {
    type Segment = str;
    type SegmentIter = Nodes<'me, str>;
    type Advance = std::future::Ready<Self>;

    fn segments(&self) -> Self::SegmentIter {
        self.base.segments()
    }

    fn advance(mut self, delta: BytesDelta) -> Self::Advance {
        let segment = self.base.str;
        let end = segment.len().min(delta.into());
        self.meter = self.meter.advance(&segment[..end]);
        self.base = self.base.advance(delta).into_inner();
        std::future::ready(self)
    }
}

impl<'me, M> RewindStream for Measured<'me, M>
where
    M: Metrics<str>,
    M::Meter: Clone,
{
    type Anchor = MeasuredAnchor<'me, M>;
    type Rewind = std::future::Ready<Self>;

    fn anchor(&self) -> Self::Anchor {
        MeasuredAnchor {
            stream: self.clone(),
        }
    }

    fn rewind(self, anchor: Self::Anchor) -> Self::Rewind {
        std::future::ready(anchor.stream)
    }
}

pub struct MeasuredAnchor<'me, M: Metrics<str>> {
    stream: Measured<'me, M>,
}

impl<'me, M: Metrics<str>> MeasuredStream for Measured<'me, M> {
    type Metrics = M;

    fn metrics(&self) -> Self::Metrics {
        self.meter.metrics()
    }
}
