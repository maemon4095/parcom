mod buffer_writer;
mod load_info;

use std::future::Future;

pub use buffer_writer::BufferWriter;
pub use load_info::LoadInfo;

pub trait SequenceBuilder<S> {
    type Length;
    type Segment: ?Sized;
    type Buffer: SequenceBuffer<Segment = Self::Segment, Length = Self::Length>;
    type Loader: SequenceLoader<Segment = Self::Segment, Length = Self::Length>;

    fn build(&self, source: S) -> (Self::Buffer, Self::Loader);
}

pub trait SequenceLoader {
    type Length;
    type Segment: ?Sized;
    type Error;
    type Load<'a>: Future<Output = Result<LoadInfo, Self::Error>>
    where
        Self: 'a;

    fn force_commit(&mut self);
    fn load(&mut self) -> Self::Load<'_>;
}

pub trait SequenceBuffer: Sized {
    type Length;
    type Segment: ?Sized;
    /// NOTE: 一度Noneを返したあとでも、nextを呼ばれる場合がある。
    type Iter<'a>: Iterator<Item = &'a Self::Segment>
    where
        Self: 'a;

    fn advance(&mut self, length: Self::Length) -> Self::Length;

    fn segments(&self) -> Self::Iter<'_>;
}

pub trait RewindSequenceBuffer: SequenceBuffer {
    type Anchor;

    fn anchor(&self) -> Self::Anchor;
    fn rewind(&mut self, anchor: Self::Anchor);
}

pub trait SequenceSource: Sized {
    type Item;
    type Error;
    type Next<'a, C>: Future<Output = C::Result>
    where
        Self: 'a,
        C: 'a + SequenceControl<Item = Self::Item, Error = Self::Error>;

    fn next<'a, C>(&'a mut self, control: C, size_hint: usize) -> Self::Next<'a, C>
    where
        C: 'a + SequenceControl<Item = Self::Item, Error = Self::Error>;
}

pub trait SequenceControl {
    type Item;
    type Result;
    type Error;
    type Writer: BufferWriter<Item = Self::Item, Result = Self::Result, Error = Self::Error>;

    fn request_writer(self, min_capacity: usize) -> Self::Writer;
    fn cancel(self, err: Self::Error) -> Self::Result;
    fn finish(self) -> Self::Result;
}
