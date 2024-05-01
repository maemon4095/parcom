use parcom_core::{IntoMeasured, MeasuredStream, Metrics, ParcomStream, RewindStream};

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
    fn segments(&self) -> impl Iterator<Item = &Self::Segment> {
        std::iter::once(self.slice)
    }

    fn advance(mut self, count: usize) -> Self {
        self.slice = &self.slice[count..];
        self
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

    fn into_measured_with(self, metrics: M) -> Self::Measured {
        Measured {
            metrics,
            base: self,
        }
    }
}

#[derive(Debug)]
pub struct Measured<'me, T, M>
where
    M: Metrics<[T]>,
{
    metrics: M,
    base: SliceStream<'me, T>,
}

impl<'me, T, M> Clone for Measured<'me, T, M>
where
    M: Metrics<[T]> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            metrics: self.metrics.clone(),
            base: self.base.clone(),
        }
    }
}

impl<'me, T, M> ParcomStream for Measured<'me, T, M>
where
    M: Metrics<[T]>,
{
    type Segment = <SliceStream<'me, T> as ParcomStream>::Segment;

    fn segments(&self) -> impl Iterator<Item = &Self::Segment> {
        self.base.segments()
    }

    fn advance(mut self, count: usize) -> Self {
        let mut rest = count;
        for segment in self.base.segments() {
            if segment.len() >= rest {
                self.metrics = self.metrics.advance(&segment[..rest]);
                break;
            }
            self.metrics = self.metrics.advance(segment);
            rest -= segment.len();
        }
        self.base = self.base.advance(count);
        self
    }
}

impl<'me, T, M> RewindStream for Measured<'me, T, M>
where
    M: Metrics<[T]> + Clone,
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
    type Location = M::Location;
    fn location(&self) -> Self::Location {
        self.metrics.location()
    }
}
