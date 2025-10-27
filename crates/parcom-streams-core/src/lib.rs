use std::future::IntoFuture;

pub trait StreamSource: Sized {
    type Segment: ?Sized;
    type Error;
    type Next<'a, C>: IntoFuture<Output = C::Response>
    where
        Self: 'a,
        C: 'a + StreamControl<Segment = Self::Segment, Error = Self::Error>;

    fn next<'a, C>(&'a mut self, control: C, size_hint: usize) -> Self::Next<'a, C>
    where
        C: 'a + StreamControl<Segment = Self::Segment, Error = Self::Error>;
}

pub trait StreamControl {
    type Segment: ?Sized;
    type Response;
    type Error;
    type Request: BufferRequest<
        Segment = Self::Segment,
        Response = Self::Response,
        Error = Self::Error,
    >;

    fn request_buffer(self, min_size: usize) -> Self::Request;
    fn cancel(self, err: Self::Error) -> Self::Response;
    fn finish(self) -> Self::Response;
}

pub trait BufferRequest {
    type Segment: ?Sized;
    type Response;
    type Error;

    fn buffer(&mut self) -> &mut Self::Segment;
    fn advance(self, written: usize) -> Self::Response;
    fn cancel(self, err: Self::Error) -> Self::Response;
}
