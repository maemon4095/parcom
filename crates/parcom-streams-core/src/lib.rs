use std::future::Future;

pub trait StreamSource: Sized {
    type Segment: ?Sized;
    type Error;
    type Next<'a, C: StreamControl<Self>>: Future<Output = C::Response>
    where
        Self: 'a;
    fn next<C: StreamControl<Self>>(&mut self, control: C, size_hint: usize) -> Self::Next<'_, C>;
}

pub trait StreamControl<S: StreamSource> {
    type Response;
    type Request: BufferRequest<S, Response = Self::Response>;

    fn request_buffer(self, min_size: usize) -> Self::Request;
    fn cancel(self, err: S::Error) -> Self::Response;
    fn finish(self) -> Self::Response;
}

pub trait BufferRequest<S: StreamSource> {
    type Response;

    fn buffer(&mut self) -> &mut S::Segment;
    fn advance(self, written: usize) -> Self::Response;
    fn cancel(self, err: S::Error) -> Self::Response;
}
