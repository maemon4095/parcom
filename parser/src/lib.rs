#[cfg(feature = "standard")]
mod standard_extension;
#[cfg(feature = "streams")]
mod streams;
pub trait Parser<T> {
    type Output;
    type Error;

    fn parse<S: ParseStream<Item = T>>(
        &self,
        input: S,
    ) -> Result<(Self::Output, S), (Self::Error, S)>;
}

pub trait ParseStream: Stream {
    type Location: Ord;
    fn location(&self, index: usize) -> Self::Location;
}

pub trait Stream {
    type Item;
    type Anchor;
    type Iter<'a>: 'a + Iterator<Item = &'a [Self::Item]>
    where
        Self: 'a;

    fn segments(&self) -> Self::Iter<'_>;
    fn anchor(&self) -> Self::Anchor;
    // AnchorとSelfの組で返しても良いが，バックトラックが不要な場合にパフォーマンスが低下する．
    fn advance(self, count: usize) -> Self;
    fn rewind(self, anchor: Self::Anchor) -> Self;
}

mod internal {
    pub trait Sealed {}

    impl<T> Sealed for T {}
}
