use std::marker::PhantomData;

use parcom_sequence_core::{BufferWriter, SequenceControl};

use crate::BufferStrategy;

pub struct Control<'a, T, E, S: BufferStrategy> {
    buf: &'a mut Vec<T>,
    strategy: &'a S,
    _phantom: PhantomData<fn() -> E>,
}

impl<'a, T, E, S: BufferStrategy> Control<'a, T, E, S> {
    pub(super) fn new(buf: &'a mut Vec<T>, strategy: &'a S) -> Self {
        Self {
            buf,
            strategy,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T, E, S: BufferStrategy> SequenceControl for Control<'a, T, E, S> {
    type Item = T;
    type Result = Response<T, Self::Error>;
    type Error = E;
    type Writer = Request<'a, T, E>;

    fn request_writer(self, min_size: usize) -> Self::Writer {
        let buf = self.buf;
        let offset = buf.len();
        let spare_capacity = buf.capacity() - offset;

        if spare_capacity >= min_size {
            Request {
                to_append: Vec::new(),
                buf,
                offset,
                _phantom: PhantomData,
            }
        } else {
            let cap = self.strategy.calc_capacity(min_size);
            let to_append = std::mem::replace(buf, Vec::with_capacity(cap));
            Request {
                to_append,
                buf,
                offset: 0,
                _phantom: PhantomData,
            }
        }
    }

    fn cancel(self, err: Self::Error) -> Self::Result {
        Response::Cancel(err)
    }

    fn finish(self) -> Self::Result {
        Response::Finish(std::mem::replace(self.buf, Vec::new()))
    }
}

pub enum Response<T, E> {
    Appended { len: usize, cap: usize },
    Advance { buf: Vec<T>, len: usize, cap: usize },
    Finish(Vec<T>),
    Cancel(E),
}

pub struct Request<'a, T, E> {
    to_append: Vec<T>,
    buf: &'a mut Vec<T>,
    offset: usize,
    _phantom: PhantomData<fn() -> E>,
}

impl<'a, T, E> BufferWriter for Request<'a, T, E> {
    type Segment = [T];
    type Item = T;
    type Result = Response<T, E>;
    type Error = E;

    fn capacity(&self) -> usize {
        self.buf.capacity() - self.buf.len()
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
        if self.to_append.is_empty() {
            Response::Appended {
                cap: self.buf.capacity(),
                len: self.buf.len(),
            }
        } else {
            Response::Advance {
                buf: self.to_append,
                cap: self.buf.capacity(),
                len: self.buf.len(),
            }
        }
    }

    fn cancel(self, err: Self::Error) -> Self::Result {
        self.buf.drain(self.offset..);
        Response::Cancel(err)
    }
}
