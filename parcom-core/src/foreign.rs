use crate::{RewindStream, Stream};

impl Stream for &str {
    type Segment = str;

    type Iter<'a> = std::iter::Once<&'a str>
    where
        Self: 'a;

    fn segments(&self) -> Self::Iter<'_> {
        std::iter::once(self)
    }

    fn advance(self, count: usize) -> Self {
        let mut chars = self.chars();
        for _ in 0..count {
            chars.next();
        }
        chars.as_str()
    }
}

impl RewindStream for &str {
    type Anchor = Self;

    fn anchor(&self) -> Self::Anchor {
        self
    }

    fn rewind(self, anchor: Self::Anchor) -> Self {
        anchor
    }
}
