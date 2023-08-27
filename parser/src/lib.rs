#[cfg(feature = "standard")]
mod standard_extension;
#[cfg(feature = "streams")]
mod streams;
pub trait Parser<S: ParseStream> {
    type Output;
    type Error;

    fn parse(&self, input: S) -> Result<(Self::Output, S), (Self::Error, S)>;
}

impl<S: ParseStream, O, E, F: Fn(S) -> Result<(O, S), (E, S)>> Parser<S> for F {
    type Output = O;
    type Error = E;

    fn parse(&self, input: S) -> Result<(Self::Output, S), (Self::Error, S)> {
        self(input)
    }
}

pub trait ParseStream: RewindStream {
    type Location: Ord;
    fn location(&self, index: usize) -> Self::Location;
}

pub trait RewindStream: Stream {
    type Anchor;

    fn anchor(&self) -> Self::Anchor;
    fn rewind(self, anchor: Self::Anchor) -> Self;
}

pub trait Stream {
    type Segment: ?Sized;
    type Iter<'a>: 'a + Iterator<Item = &'a Self::Segment>
    where
        Self: 'a;

    fn segments(&self) -> Self::Iter<'_>;
    fn advance(self, count: usize) -> Self;
}

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

mod internal {
    pub trait Sealed {}

    impl<T> Sealed for T {}
}
