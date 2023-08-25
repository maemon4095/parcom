mod from_read;
pub trait Parser<T> {
    type Output;
    type Error;

    fn parse<S: ParseStream<T>>(&self, input: S) -> Result<(Self::Output, S), Self::Error>;
}

pub trait ParseStream<T>: Stream<T> {
    type Location;
    fn location(&self, index: usize) -> Self::Location;
}

pub trait Stream<T> {
    type Anchor;
    type Iter<'a>: 'a + Iterator<Item = &'a [T]>
    where
        Self: 'a,
        T: 'a;

    fn segments(&self) -> Self::Iter<'_>;
    fn anchor(&self) -> Self::Anchor;
    // AnchorとSelfの組で返しても良いが，バックトラックが不要な場合にパフォーマンスが低下する．
    fn advance(self, count: usize) -> Self;
    fn rewind(self, anchor: Self::Anchor) -> Self;
}
