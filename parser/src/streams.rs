use std::io::Read;

mod buffer_writer;
mod from_source;

pub trait SegmentFactory<T> {
    type Segment;
    fn stage(&mut self) -> buffer_writer::BufferStaging<'_, T>;
    fn complete(self) -> Self::Segment;
}

pub trait StreamSource<T> {
    fn request<B: BufferWriter<T> + ?Sized>(&mut self, writer: &mut B) -> SourceResponse;
}

struct ReadSouce<T: Read> {
    source: T,
}

impl<T: Read> StreamSource<u8> for ReadSouce<T> {
    fn request<B: BufferWriter<u8> + ?Sized>(&mut self, writer: &mut B) -> SourceResponse {
        let mut staging = writer.stage(1024);
        let caps = staging.spare_capacity_mut();
        todo!()
    }
}

pub struct SourceResponse {
    pub is_completed: bool,
}
