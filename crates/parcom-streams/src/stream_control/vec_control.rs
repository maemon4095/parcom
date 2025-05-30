use std::marker::PhantomData;

use parcom_streams_core::{BufferRequest, StreamControl};

use super::Response;

pub struct VecControl<'a, T: Default, E> {
    buf: &'a mut Vec<T>,
    _phantom: PhantomData<E>,
}

impl<'a, T: Default, E> VecControl<'a, T, E> {
    pub fn new(buf: &'a mut Vec<T>) -> Self {
        Self {
            buf,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: Default, E> StreamControl for VecControl<'a, T, E> {
    type Segment = [T];
    type Response = Response<(), Self::Error>;
    type Error = E;
    type Request = Request<'a, T, E>;

    fn request_buffer(self, min_size: usize) -> Self::Request {
        let offset = self.buf.len();
        let buf = self.buf;

        buf.extend(std::iter::repeat_with(Default::default).take(min_size));

        Request {
            offset,
            buf,
            _phantom: PhantomData,
        }
    }

    fn cancel(self, err: Self::Error) -> Self::Response {
        Response::Cancel((), err)
    }

    fn finish(self) -> Self::Response {
        Response::Finish(())
    }
}

pub struct Request<'a, T: Default, E> {
    offset: usize,
    buf: &'a mut Vec<T>,
    _phantom: PhantomData<E>,
}

impl<'a, T: Default, E> BufferRequest for Request<'a, T, E> {
    type Control = VecControl<'a, T, E>;

    fn buffer(&mut self) -> &mut <Self::Control as StreamControl>::Segment {
        &mut self.buf[self.offset..]
    }

    fn advance(self, written: usize) -> <Self::Control as StreamControl>::Response {
        self.buf.drain((self.offset + written)..);
        Response::Advance(())
    }

    fn cancel(
        self,
        err: <Self::Control as StreamControl>::Error,
    ) -> <Self::Control as StreamControl>::Response {
        self.buf.drain(self.offset..);
        Response::Cancel((), err)
    }
}
