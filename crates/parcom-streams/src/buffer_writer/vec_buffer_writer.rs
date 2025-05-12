pub struct VecBufferWriter<T: Default> {
    buf: Vec<T>,
}

impl<T: Default> VecBufferWriter<T> {
    pub fn new(buf: Vec<T>) -> Self {
        Self { buf }
    }

    pub fn request_buffer(self, min_size: usize) -> Request<T> {
        let offset = self.buf.len();
        let mut buf = self.buf;

        buf.extend(std::iter::repeat_with(Default::default).take(min_size));

        Request { offset, buf }
    }

    pub fn finish(self) -> Vec<T> {
        self.buf
    }
}

pub struct Request<T: Default> {
    offset: usize,
    buf: Vec<T>,
}

impl<T: Default> Request<T> {
    pub fn buffer(&mut self) -> &mut [T] {
        &mut self.buf[self.offset..]
    }

    pub fn advance(mut self, written: usize) -> Vec<T> {
        self.buf.drain((self.offset + written)..);
        self.buf
    }

    pub fn cancel(mut self) -> Vec<T> {
        self.buf.drain(self.offset..);
        self.buf
    }
}
