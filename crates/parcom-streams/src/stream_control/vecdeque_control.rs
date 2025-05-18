use std::{collections::VecDeque, marker::PhantomData};

use parcom_streams_core::{BufferRequest, StreamControl};

use super::Response;

pub struct VecDequeControl<T: Default, E> {
    buf: VecDeque<T>,
    _phantom: PhantomData<fn(E) -> E>,
}

impl<T: Default, E> VecDequeControl<T, E> {
    pub fn new(buf: VecDeque<T>) -> Self {
        Self {
            buf,
            _phantom: PhantomData,
        }
    }
}

impl<T: Default, E> StreamControl for VecDequeControl<T, E> {
    type Segment = [T];
    type Response = Response<VecDeque<T>, E>;
    type Error = E;
    type Request = Request<T, E>;

    fn request_buffer(self, min_size: usize) -> Self::Request {
        let offset = self.buf.len();
        let mut buf = self.buf;

        buf.extend(std::iter::repeat_with(Default::default).take(min_size));

        let (left, _) = buf.as_slices();

        if offset < left.len() && (left.len() - offset) < min_size {
            buf.make_contiguous();
        }

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
    buf: VecDeque<T>,
    _phantom: PhantomData<fn(E) -> E>,
}

impl<T: Default, E> BufferRequest for Request<T, E> {
    type Control = VecDequeControl<T, E>;

    fn buffer(&mut self) -> &mut [T] {
        let (left, right) = self.buf.as_mut_slices();
        let offset = self.offset;

        if offset < left.len() {
            &mut left[offset..]
        } else {
            &mut right[(offset - left.len())..]
        }
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
