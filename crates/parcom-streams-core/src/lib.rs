use std::future::Future;

pub trait StreamSource: Sized {
    type Segment: ?Sized;
    type Error;
    type Next<'a, C>: Future<Output = C::Response>
    where
        Self: 'a,
        C: StreamControl<Segment = Self::Segment, Error = Self::Error>;

    fn next<C>(&mut self, control: C, size_hint: usize) -> Self::Next<'_, C>
    where
        C: StreamControl<Segment = Self::Segment, Error = Self::Error>;
}

pub trait StreamControl {
    type Segment: ?Sized;
    type Response;
    type Error;
    type Request: BufferRequest<Control = Self>;

    fn request_buffer(self, min_size: usize) -> Self::Request;
    fn cancel(self, err: Self::Error) -> Self::Response;
    fn finish(self) -> Self::Response;
}

pub trait BufferRequest {
    type Control: StreamControl;

    fn buffer(&mut self) -> &mut <Self::Control as StreamControl>::Segment;
    fn advance(self, written: usize) -> <Self::Control as StreamControl>::Response;
    fn cancel(
        self,
        err: <Self::Control as StreamControl>::Error,
    ) -> <Self::Control as StreamControl>::Response;
}
