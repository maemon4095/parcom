use std::marker::PhantomData;

pub struct VecBufferWriter<T: Default> {
    _phantom: PhantomData<T>,
}

impl<T: Default> VecBufferWriter<T> {
    pub fn request_buffer(self, min_size: usize) -> Request<T> {
        Request {
            buf: Vec::from_iter(std::iter::repeat_with(Default::default).take(min_size)),
        }
    }
}

pub struct Request<T: Default> {
    buf: Vec<T>,
}

impl<T: Default> Request<T> {
    pub fn advance(mut self, written: usize) -> Vec<T> {
        self.buf.drain((written + 1)..);
        self.buf
    }
}
