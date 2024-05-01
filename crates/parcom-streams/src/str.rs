use parcom_core::{IntoMeasured, MeasuredStream, Metrics, ParcomStream, RewindStream};

#[derive(Debug, Clone)]
pub struct StrCharStream<'me> {
    str: &'me str,
}

impl<'me> StrCharStream<'me> {
    pub fn new(str: &'me str) -> Self {
        Self { str }
    }
}

impl<'me> ParcomStream for StrCharStream<'me> {
    type Segment = str;

    fn segments(&self) -> impl Iterator<Item = &Self::Segment> {
        std::iter::once(self.str)
    }

    fn advance(mut self, count: usize) -> Self {
        let mut chars = self.str.chars();
        for _ in 0..count {
            chars.next();
        }
        self.str = chars.as_str();
        self
    }
}
impl<'me> RewindStream for StrCharStream<'me> {
    type Anchor = Anchor<'me>;

    fn anchor(&self) -> Self::Anchor {
        Anchor {
            stream: self.clone(),
        }
    }

    fn rewind(self, anchor: Self::Anchor) -> Self {
        anchor.stream
    }
}

pub struct Anchor<'me> {
    stream: StrCharStream<'me>,
}

impl<'me, M> IntoMeasured<M> for StrCharStream<'me>
where
    M: Metrics<str>,
{
    type Measured = Measured<'me, M>;

    fn into_measured_with(self, metrics: M) -> Self::Measured {
        Measured {
            metrics,
            base: self,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Measured<'me, M: Metrics<str>> {
    metrics: M,
    base: StrCharStream<'me>,
}

impl<'me, M: Metrics<str>> ParcomStream for Measured<'me, M> {
    type Segment = str;

    fn segments(&self) -> impl Iterator<Item = &Self::Segment> {
        self.base.segments()
    }

    fn advance(mut self, count: usize) -> Self {
        let mut rest = count;
        for segment in self.base.segments() {
            let count = segment.chars().count();
            if count >= rest {
                self.metrics = self.metrics.advance(&segment[..rest]);
                break;
            }

            self.metrics = self.metrics.advance(segment);
            rest -= count;
        }
        self.base = self.base.advance(count);
        self
    }
}

impl<'me, M> RewindStream for Measured<'me, M>
where
    M: Metrics<str> + Clone,
{
    type Anchor = MeasuredAnchor<'me, M>;

    fn anchor(&self) -> Self::Anchor {
        MeasuredAnchor {
            stream: self.clone(),
        }
    }

    fn rewind(self, anchor: Self::Anchor) -> Self {
        anchor.stream
    }
}

pub struct MeasuredAnchor<'me, M: Metrics<str>> {
    stream: Measured<'me, M>,
}

impl<'me, M: Metrics<str>> MeasuredStream for Measured<'me, M> {
    type Location = M::Location;

    fn location(&self) -> Self::Location {
        self.metrics.location()
    }
}
