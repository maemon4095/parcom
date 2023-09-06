#[cfg(feature = "foreign")]
pub mod foreign;
pub type ParseResult<S, O, E> = Result<(O, S), (E, S)>;

pub trait Parser<S> {
    type Output;
    type Error;

    fn parse(&self, input: S) -> ParseResult<S, Self::Output, Self::Error>;
}

pub trait Parse<S>: Sized {
    type Error;
    fn parse(input: S) -> ParseResult<S, Self, Self::Error>;
}

impl<S, O, E, F: Fn(S) -> Result<(O, S), (E, S)>> Parser<S> for F {
    type Output = O;
    type Error = E;

    fn parse(&self, input: S) -> ParseResult<S, Self::Output, Self::Error> {
        self(input)
    }
}

impl<S, O, E> Parser<S> for Box<dyn Parser<S, Output = O, Error = E>> {
    type Output = O;
    type Error = E;

    fn parse(&self, input: S) -> Result<(Self::Output, S), (Self::Error, S)> {
        self.as_ref().parse(input)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Delta {
    Positive(usize),
    Negative(usize),
}

impl Delta {
    pub fn abs(self) -> usize {
        match self {
            Self::Positive(n) | Self::Negative(n) => n,
        }
    }
}

pub trait Location: Ord {
    /// return self - rhs
    fn delta(&self, rhs: &Self) -> Delta;
}

pub trait ParseStream: RewindStream {
    type Location: Location;
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
