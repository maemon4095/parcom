use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{IterativeParserOnce, ParserOnce, Sequence, SequenceSegment, SequenceSource};

// runnerがsequenceを内部で使う形になる。
// runnerとsequenceの実装を分けたいけど、うまい抽象化はあるか？
// sourceはsliceしか生成できない。strとかを扱うためには別の手段が必要。
pub trait ParseRunner {
    type Sequence<S: SequenceSource>: Sequence;
    type Parse<P, S>: Future<Output = Result<P::Output, RunnerError<P::Error, S::Error>>>
    where
        P: ParserOnce<Self::Sequence<S>>,
        S: SequenceSource;

    type IterationSession<P, S>: IterativeParseSession<
        Output = P::Output,
        Error = RunnerError<P::Error, S::Error>,
    >
    where
        P: IterativeParserOnce<Self::Sequence<S>>,
        S: SequenceSource;

    fn parse<P, S>(&self, parser: P, source: S) -> Self::Parse<P, S>
    where
        P: ParserOnce<Self::Sequence<S>>,
        S: SequenceSource;

    fn start_parse<P, S>(&self, parser: P, source: S) -> Self::IterationSession<P, S>
    where
        P: IterativeParserOnce<Self::Sequence<S>>,
        S: SequenceSource;
}

pub enum RunnerError<P, S> {
    Parser(P),
    Io(S),
}

pub trait IterativeParseSession {
    type Output;
    type Error;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<Self::Output>, Self::Error>>;
}

trait SequenceLoader {
    type Repr: ResumableIterable<Cursor = Self::Cursor, Item = Self::Segment>;
    type Cursor;
    type Error;
    type Segment: ?Sized;
    type Append: Future<Output = Result<Self::Repr, Self::Error>>;

    fn append(&mut self, repr: Self::Repr) -> Self::Append;
}

trait SequenceInit {
    type Cursor;
    type Error;
    type Segment: ?Sized;
    type Repr: ResumableIterable<Cursor = Self::Cursor, Item = Self::Segment>;
    type Loader: SequenceLoader<
        Repr = Self::Repr,
        Segment = Self::Segment,
        Cursor = Self::Cursor,
        Error = Self::Error,
    >;

    fn init(self) -> (Self::Repr, Self::Loader);
}

trait ResumableIterator {
    type Cursor;
    type Item: ?Sized;

    fn next(&self, cursor: Self::Cursor) -> (Self::Cursor, Option<&Self::Item>);
}

trait ResumableIterable {
    type Cursor;
    type Item: ?Sized;
    type Iter: ResumableIterator<Cursor = Self::Cursor, Item = Self::Item>;

    fn iter(&self) -> (Self::Cursor, Self::Iter);
}
