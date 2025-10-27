use std::{collections::VecDeque, marker::PhantomData};

use parcom_streams_core::{BufferRequest, StreamControl};

use super::Response;

pub struct VecDequeControl<'a, T: Default, E> {
    buf: &'a mut VecDeque<T>,
    _phantom: PhantomData<fn(E) -> E>,
}

impl<'a, T: Default, E> VecDequeControl<'a, T, E> {
    pub fn new(buf: &'a mut VecDeque<T>) -> Self {
        Self {
            buf,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: Default, E> StreamControl for VecDequeControl<'a, T, E> {
    type Segment = [T];
    type Response = Response<(), E>;
    type Error = E;
    type Request = Request<'a, T, E>;

    fn request_buffer(self, min_size: usize) -> Self::Request {
        let offset = self.buf.len();
        let buf = self.buf;

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
        Response::Cancel((), err)
    }

    fn finish(self) -> Self::Response {
        Response::Finish(())
    }
}

pub struct Request<'a, T: Default, E> {
    offset: usize,
    buf: &'a mut VecDeque<T>,
    _phantom: PhantomData<fn(E) -> E>,
}

impl<'a, T: Default, E> BufferRequest for Request<'a, T, E> {
    type Segment = [T];
    type Response = Response<(), E>;
    type Error = E;

    fn buffer(&mut self) -> &mut [T] {
        let (left, right) = self.buf.as_mut_slices();
        let offset = self.offset;

        if offset < left.len() {
            &mut left[offset..]
        } else {
            &mut right[(offset - left.len())..]
        }
    }

    fn advance(self, written: usize) -> Self::Response {
        self.buf.drain((self.offset + written)..);
        Response::Advance(())
    }

    fn cancel(self, err: Self::Error) -> Self::Response {
        self.buf.drain(self.offset..);
        Response::Cancel((), err)
    }
}
