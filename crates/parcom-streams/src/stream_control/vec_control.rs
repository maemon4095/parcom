use std::marker::PhantomData;

use parcom_streams_core::{BufferWriter, StreamControl};

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
    type Item = T;
    type Result = Response<(), Self::Error>;
    type Error = E;
    type Writer = Request<'a, T, E>;

    fn request_writer(self, min_size: usize) -> Self::Writer {
        let offset = self.buf.len();
        let buf = self.buf;

        buf.reserve(min_size);

        Request {
            offset,
            buf,
            _phantom: PhantomData,
        }
    }

    fn cancel(self, err: Self::Error) -> Self::Result {
        Response::Cancel((), err)
    }

    fn finish(self) -> Self::Result {
        Response::Finish(())
    }
}

pub struct Request<'a, T: Default, E> {
    offset: usize,
    buf: &'a mut Vec<T>,
    _phantom: PhantomData<E>,
}

impl<'a, T: Default, E> BufferWriter for Request<'a, T, E> {
    type Item = T;
    type Result = Response<(), E>;
    type Error = E;

    fn capacity(&self) -> usize {
        self.buf.capacity() - self.offset
    }

    fn len(&self) -> usize {
        self.buf.len() - self.offset
    }

    fn as_ptr(&self) -> *const Self::Item {
        unsafe { self.buf.as_ptr().add(self.offset) }
    }

    fn as_mut_ptr(&mut self) -> *mut Self::Item {
        unsafe { self.buf.as_mut_ptr().add(self.offset) }
    }

    unsafe fn set_len(&mut self, new_len: usize) {
        self.buf.set_len(new_len + self.offset);
    }

    fn advance(self) -> Self::Result {
        Response::Advance(())
    }

    fn cancel(self, err: Self::Error) -> Self::Result {
        self.buf.drain(self.offset..);
        Response::Cancel((), err)
    }
}
