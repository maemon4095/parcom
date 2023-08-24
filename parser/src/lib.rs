mod from_read;
pub trait Parser<T> {
    type Output;
    type Error;

    fn parse<S: Sequence<T>>(&self, input: S) -> Result<(Self::Output, S), Self::Error>;
}

pub trait Sequence<T> {
    type Segment: AsRef<[T]>;
    type Iter: Iterator<Item = Self::Segment>;

    fn segments(&self) -> Self::Iter;
}

// エラー表示のために，文字数，行数などを知りたい．
// しかし，チェックポイントには不要である．
// また，位置情報をadvance, rewind以外の方法で取得したい．

// バックトラックに対応するために，必要なセグメントのみ保持したい．
// Anchorがdropされていなければaliveで良いか．
pub trait ParseStream<T> {
    type Location;
    type Anchor;
    type Segments<'a>: Iterator<Item = &'a [T]>
    where
        Self: 'a,
        T: 'a;

    fn segments(&self) -> Self::Segments<'_>;
    fn location(&self, indes: usize) -> Self::Location;
    fn anchor(&self) -> Self::Anchor;
    // AnchorとSelfの組で返しても良いが，バックトラックが不要な場合にパフォーマンスが低下する．
    fn advance(self, count: usize) -> Self;
    fn rewind(self, anchor: Self::Anchor) -> Self;
}
