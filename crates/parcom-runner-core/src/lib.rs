use parcom_core::{IterativeParserOnce, ParserOnce, Sequence};
use parcom_sequence_core::SequenceSource;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub trait ParseRunner<S: SequenceSource> {
    type Sequence: Sequence;
    type Parse<P>: Future<Output = Result<P::Output, RunnerError<P::Error, S::Error>>>
    where
        P: ParserOnce<Self::Sequence>,
        S: SequenceSource;

    type IterationSession<P>: IterativeParseSession<
        Output = P::Output,
        Error = RunnerError<P::Error, S::Error>,
    >
    where
        P: IterativeParserOnce<Self::Sequence>,
        S: SequenceSource;

    fn parse<P>(&self, parser: P, source: S) -> Self::Parse<P>
    where
        P: ParserOnce<Self::Sequence>;

    fn start_parse<P>(&self, parser: P, source: S) -> Self::IterationSession<P>
    where
        P: IterativeParserOnce<Self::Sequence>,
        S: SequenceSource;
}

pub enum RunnerError<P, S> {
    Parser(P),
    Source(S),
}

pub trait IterativeParseSession {
    type Output;
    type Error;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<Self::Output>, Self::Error>>;
}
