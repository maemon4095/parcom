use std::collections::VecDeque;

pub struct VecDequeBufferWriter<T: Default> {
    buf: VecDeque<T>,
}

impl<T: Default> VecDequeBufferWriter<T> {
    pub fn new(buf: VecDeque<T>) -> Self {
        Self { buf }
    }

    pub fn request_buffer(self, min_size: usize) -> Request<T> {
        let offset = self.buf.len();
        let mut buf = self.buf;

        buf.extend(std::iter::repeat_with(Default::default).take(min_size));

        let (left, _) = buf.as_slices();

        if offset < left.len() && (left.len() - offset) < min_size {
            buf.make_contiguous();
        }

        Request { offset, buf }
    }

    pub fn finish(self) -> VecDeque<T> {
        self.buf
    }
}

pub struct Request<T: Default> {
    offset: usize,
    buf: VecDeque<T>,
}

impl<T: Default> Request<T> {
    pub fn buffer(&mut self) -> &mut [T] {
        let (left, right) = self.buf.as_mut_slices();
        let offset = self.offset;

        if offset < left.len() {
            &mut left[offset..]
        } else {
            &mut right[(offset - left.len())..]
        }
    }

    pub fn advance(mut self, written: usize) -> VecDeque<T> {
        self.buf.drain((self.offset + written)..);
        self.buf
    }

    pub fn cancel(mut self) -> VecDeque<T> {
        self.buf.drain(self.offset..);
        self.buf
    }
}
