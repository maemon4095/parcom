use crate::ParcomStream;

pub trait Metrics<S: ?Sized> {
    type Location;

    fn advance(self, segment: &S) -> Self;
    fn location(&self) -> Self::Location;
}

pub trait MeasuredStream: ParcomStream {
    type Location;
    fn location(&self) -> Self::Location;
}

pub trait IntoMeasured<M: Metrics<Self::Segment>>: ParcomStream {
    type Measured: MeasuredStream;

    fn into_measured(self) -> Self::Measured
    where
        M: Default,
    {
        self.into_measured_with(M::default())
    }

    fn into_measured_with(self, metrics: M) -> Self::Measured;
}
