use crate::{ParseStream, Parser, RewindStream, Stream};
mod slice;
mod str;

pub use slice::SliceStream;
pub use str::StrStream;

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
        chars.nth(count);
        chars.as_str()
    }
}

impl<T> Stream for &[T] {
    type Segment = [T];
    type Iter<'a> = std::iter::Once<&'a [T]>
    where
        Self: 'a;

    fn segments(&self) -> Self::Iter<'_> {
        std::iter::once(self)
    }

    fn advance(self, count: usize) -> Self {
        &self[count..]
    }
}

impl<T> RewindStream for &[T] {
    type Anchor = Self;

    fn anchor(&self) -> Self::Anchor {
        self
    }

    fn rewind(self, anchor: Self::Anchor) -> Self {
        anchor
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

impl<S: ParseStream, O, E, F: Fn(S) -> Result<(O, S), (E, S)>> Parser<S> for F {
    type Output = O;
    type Error = E;

    fn parse(&self, input: S) -> Result<(Self::Output, S), (Self::Error, S)> {
        self(input)
    }
}

impl<S: ParseStream, O, E> Parser<S> for Box<dyn Parser<S, Output = O, Error = E>> {
    type Output = O;
    type Error = E;

    fn parse(&self, input: S) -> Result<(Self::Output, S), (Self::Error, S)> {
        self.as_ref().parse(input)
    }
}

#[cfg(test)]
mod test {
    use crate::{ParseStream, Stream};

    use super::StrStream;

    #[test]
    fn strstream_location() {
        let stream = StrStream::new("abc\n\ndef");

        let idx = 9;
        let loc = stream.location(idx);
        let c = stream.segments().flat_map(|s| s.chars()).nth(idx);
        println!("{:?} @ {:?}", c, loc);
    }
}
