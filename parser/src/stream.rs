use std::io::Read;
pub mod buffer_writer;
mod from_source;

pub use from_source::FromSource;

pub trait SegmentFactory<T> {
    type SegmentConstruct;

    fn alloc(&mut self, min_capacity: usize) -> (*mut T, usize);
    fn complete(&mut self, len: usize) -> Self::SegmentConstruct;
}

pub trait StreamSource<T> {
    fn request<F: SegmentFactory<T> + ?Sized>(
        &mut self,
        factory: &mut F,
    ) -> Option<F::SegmentConstruct>;
}

pub struct ReadSouce<T: Read> {
    source: T,
}

impl<T: Read> StreamSource<u8> for ReadSouce<T> {
    fn request<F: SegmentFactory<u8> + ?Sized>(
        &mut self,
        factory: &mut F,
    ) -> Option<F::SegmentConstruct> {
        let mut buf = [0; 1024];
        let written = self.source.read(&mut buf).unwrap();
        if written == 0 {
            return None;
        }

        let (ptr, capacity) = factory.alloc(written);

        assert!(capacity >= written);

        unsafe { std::ptr::copy_nonoverlapping(buf.as_ptr(), ptr, written) };

        Some(factory.complete(written))
    }
}
