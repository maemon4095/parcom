use std::marker::PhantomData;

use parcom_streams_core::{BufferRequest, StreamControl};

use super::Response;

pub struct VecControl<T: Default, E> {
    buf: Vec<T>,
    _phantom: PhantomData<E>,
}

impl<T: Default, E> VecControl<T, E> {
    pub fn new(buf: Vec<T>) -> Self {
        Self {
            buf,
            _phantom: PhantomData,
        }
    }
}

impl<T: Default, E> StreamControl for VecControl<T, E> {
    type Segment = [T];
    type Response = Response<Vec<T>, Self::Error>;
    type Error = E;
    type Request = Request<T, E>;

    fn request_buffer(self, min_size: usize) -> Self::Request {
        let offset = self.buf.len();
        let mut buf = self.buf;

        buf.extend(std::iter::repeat_with(Default::default).take(min_size));

        Request {
            offset,
            buf,
            _phantom: PhantomData,
        }
    }

    fn cancel(self, err: Self::Error) -> Self::Response {
        Response::Cancel(self.buf, err)
    }

    fn finish(self) -> Self::Response {
        Response::Finish(self.buf)
    }
}

pub struct Request<T: Default, E> {
    offset: usize,
    buf: Vec<T>,
    _phantom: PhantomData<E>,
}

impl<T: Default, E> BufferRequest for Request<T, E> {
    type Control = VecControl<T, E>;

    fn buffer(&mut self) -> &mut <Self::Control as StreamControl>::Segment {
        &mut self.buf[self.offset..]
    }

    fn advance(mut self, written: usize) -> <Self::Control as StreamControl>::Response {
        self.buf.drain((self.offset + written)..);
        Response::Advance(self.buf)
    }

    fn cancel(
        mut self,
        err: <Self::Control as StreamControl>::Error,
    ) -> <Self::Control as StreamControl>::Response {
        self.buf.drain(self.offset..);
        Response::Cancel(self.buf, err)
    }
}
