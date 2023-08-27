mod from_read;
mod from_source;

pub trait BufferWriter<T> {
    fn rent(&mut self, capacity: usize) -> &mut [T];
    fn advance(&mut self, count: usize); // mut referenceは唯一であるから，このAPIで問題ない．
}

pub trait StreamSource<T> {
    fn request<B: BufferWriter<T> + ?Sized>(&mut self, writer: &mut B) -> SourceResponse;
}

pub struct SourceResponse {
    pub is_completed: bool,
}
