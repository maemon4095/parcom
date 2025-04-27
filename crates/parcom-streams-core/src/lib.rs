use std::future::Future;

pub enum IterationStep {
    NonTerminal,
    Terminal,
}

pub trait StreamSource<T> {
    type Error;
    type Next<W: BufferWriter<T>>: Future<Output = Result<IterationStep, Self::Error>>;
    fn next<W: BufferWriter<T>>(&mut self, writer: W, size_hint: usize) -> Self::Next<W>;
}

pub trait BufferWriter<T> {
    type Request: BufferRequest<T>;
    fn request_buffer(self, min_size: usize) -> Self::Request;
}

pub trait BufferRequest<T> {
    fn buffer(&mut self) -> &mut [T];
    fn advance(self, written: usize);
}
