use parcom_core::Metrics;

#[derive(Debug, Clone, Copy)]
pub struct Count(usize);

impl<T> Metrics<[T]> for Count {
    type Location = usize;

    fn advance(mut self, segment: &[T]) -> Self {
        self.0 += segment.len();
        self
    }

    fn location(&self) -> Self::Location {
        self.0
    }
}
