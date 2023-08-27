use std::io::Read;

use self::buffer_writer::BufferStaging;

mod buffer_writer;
mod from_read;

pub trait SegmentFactory<T> {
    type Segment;

    fn next(&self) -> Option<Self::Segment>;
}

pub trait StreamSource<T> {
    fn request<B: ?Sized>(&mut self, writer: &mut B) -> SourceResponse;
}

struct ReadSouce<T: Read> {
    source: T,
}

impl<T: Read> StreamSource<u8> for ReadSouce<T> {
    fn request<B: ?Sized>(&mut self, writer: &mut B) -> SourceResponse {
        todo!()
    }
}

pub struct SourceResponse {
    pub is_completed: bool,
}
