use std::ops::Deref;

use parcom_core::Location;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Count(usize);

impl From<usize> for Count {
    fn from(value: usize) -> Self {
        Count(value)
    }
}

impl Deref for Count {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Count {
    fn default() -> Self {
        Self(0)
    }
}

impl<T> Location<[T]> for Count {
    fn advance(mut self, segment: &[T]) -> Self {
        self.0 += segment.len();
        self
    }
}
