use crate::Sequence;

pub trait Meter<S: ?Sized> {
    type Metrics: Metrics<S, Meter = Self>;

    fn advance(self, segment: &S) -> Self;
    fn metrics(&self) -> Self::Metrics;
}

pub trait Metrics<S: ?Sized>: Eq + Ord {
    type Meter: Meter<S, Metrics = Self>;
}

pub trait MeasuredSequence: Sequence {
    type Metrics: Metrics<Self::Segment>;
    fn metrics(&self) -> Self::Metrics;
}

pub trait IntoMeasured: Sequence {
    type Measured<M: Metrics<Self::Segment>>: MeasuredSequence<Metrics = M>;

    fn into_measured<M: Metrics<Self::Segment>>(self) -> Self::Measured<M>
    where
        M::Meter: Default,
    {
        self.into_measured_with(<M::Meter as Default>::default())
    }

    fn into_measured_with<M: Metrics<Self::Segment>>(self, meter: M::Meter) -> Self::Measured<M>;
}

pub trait Measureable<M: Metrics<Self>> {
    fn measure(&self) -> M;
}

impl<S: ?Sized + Measureable<usize>> Meter<S> for usize {
    type Metrics = usize;

    fn advance(mut self, segment: &S) -> Self {
        self += segment.measure();
        self
    }

    fn metrics(&self) -> Self::Metrics {
        *self
    }
}

impl<S: ?Sized + Measureable<usize>> Metrics<S> for usize {
    type Meter = usize;
}

impl<T> Measureable<usize> for [T] {
    fn measure(&self) -> usize {
        self.len()
    }
}
