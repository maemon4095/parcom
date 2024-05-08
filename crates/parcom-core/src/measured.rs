use crate::ParcomStream;

pub trait Meter<S: ?Sized> {
    type Metrics: Metrics<S, Meter = Self>;

    fn advance(self, segment: &S) -> Self;
    fn metrics(&self) -> Self::Metrics;
}

pub trait Metrics<S: ?Sized>: Eq + Ord {
    type Meter: Meter<S, Metrics = Self>;
}

pub trait MeasuredStream: ParcomStream {
    type Metrics: Metrics<Self::Segment>;
    fn metrics(&self) -> Self::Metrics;
}

pub trait IntoMeasured<M: Metrics<Self::Segment>>: ParcomStream {
    type Measured: MeasuredStream<Metrics = M>;

    fn into_measured(self) -> Self::Measured
    where
        M::Meter: Default,
    {
        self.into_measured_with(<M::Meter as Default>::default())
    }

    fn into_measured_with(self, meter: M::Meter) -> Self::Measured;
}

pub trait Measureable<M: Metrics<Self>> {
    fn measure(&self) -> M;
}

pub struct Counter(usize);
impl<S: ?Sized + Measureable<usize>> Meter<S> for Counter {
    type Metrics = usize;

    fn advance(mut self, segment: &S) -> Self {
        self.0 += segment.measure();
        self
    }

    fn metrics(&self) -> Self::Metrics {
        self.0
    }
}

impl<S: ?Sized + Measureable<usize>> Metrics<S> for usize {
    type Meter = Counter;
}

impl<T> Measureable<usize> for [T] {
    fn measure(&self) -> usize {
        self.len()
    }
}

impl Measureable<usize> for str {
    fn measure(&self) -> usize {
        self.chars().count()
    }
}
