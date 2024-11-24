use std::future::Future;

pub trait StreamSource: Clone {
    type Output: ?Sized;
    type Node: AsRef<Self::Output>;
    type Future: Future<Output = Option<Self::Node>>;

    fn next(&mut self) -> Self::Future {
        self.next_with_hint(0)
    }
    fn next_with_hint(&mut self, size_hint: usize) -> Self::Future;
}
