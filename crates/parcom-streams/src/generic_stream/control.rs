use std::marker::PhantomData;

use parcom_streams_core::{BufferRequest, StreamControl};

use crate::generic_stream::GenericStreamParameter;

pub(super) struct Control<'a, T, E: 'a> {
    parameter: &'a GenericStreamParameter,
    pre_allocated_buffer: &'a mut [T],
    _phantom: PhantomData<E>,
}
impl<'a, T, E: 'a> Control<'a, T, E> {
    pub(super) fn new(
        parameter: &'a GenericStreamParameter,
        pre_allocated_buffer: &'a mut [T],
    ) -> Self {
        Self {
            parameter,
            pre_allocated_buffer,
            _phantom: PhantomData,
        }
    }
}

pub(super) enum Response<T, E> {
    PreAllocated(usize),
    Allocated(Vec<T>, usize),
    Finish,
    Error(E),
}

pub(super) enum Request<'a, T, E> {
    PreAllocated(&'a mut [T], PhantomData<E>),
    Allocated(Vec<T>),
}

impl<'a, T, E> BufferRequest for Request<'a, T, E> {
    type Segment = [T];
    type Response = Response<T, E>;
    type Error = E;

    fn buffer(&mut self) -> &mut Self::Segment {
        match self {
            Request::PreAllocated(items, _) => items,
            Request::Allocated(items) => items,
        }
    }

    fn advance(self, written: usize) -> Self::Response {
        match self {
            Request::PreAllocated(items, _) => {
                assert!(written <= items.len());
                Response::PreAllocated(written)
            }
            Request::Allocated(items) => {
                assert!(written <= items.len());
                Response::Allocated(items, written)
            }
        }
    }

    fn cancel(self, err: Self::Error) -> Self::Response {
        Response::Error(err)
    }
}

impl<'a, T: Default, E: 'a> StreamControl for Control<'a, T, E> {
    type Segment = [T];
    type Response = Response<T, E>;
    type Error = E;
    type Request = Request<'a, T, E>;

    fn request_buffer(self, min_size: usize) -> Self::Request {
        if min_size <= self.pre_allocated_buffer.len() {
            Request::PreAllocated(self.pre_allocated_buffer, PhantomData)
        } else {
            Request::Allocated(
                std::iter::repeat_with(Default::default)
                    .take(self.parameter.calc_new_segment_capacity(min_size))
                    .collect(),
            )
        }
    }

    fn cancel(self, err: Self::Error) -> Self::Response {
        Response::Error(err)
    }

    fn finish(self) -> Self::Response {
        Response::Finish
    }
}
