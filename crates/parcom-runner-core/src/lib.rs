use parcom_core::{IterativeParserOnce, ParserOnce, Sequence};
use parcom_sequence_core::{SequenceLoader, SequenceSource};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub trait ParseRunner<S: SequenceSource> {
    type Error<E>;
    type Sequence: Sequence;
    type Parse<P: ParserOnce<Self::Sequence>>: Future<
        Output = Result<P::Output, Self::Error<P::Error>>,
    >;
    type ParseIterative<P: IterativeParserOnce<Self::Sequence>>: IterativeParseSession<
        Output = P::Output,
        Error = Self::Error<P::Error>,
    >;

    fn parse<P>(&self, parser: P, source: S) -> Self::Parse<P>
    where
        S: SequenceSource,
        P: ParserOnce<Self::Sequence>;

    fn parse_iterative<P>(&self, parser: P, source: S) -> Self::ParseIterative<P>
    where
        P: IterativeParserOnce<Self::Sequence>;
}

pub enum RunnerError<P, S> {
    Parser(P),
    Source(S),
}

// やはりこのあたりは複雑になりすぎている。具象型のchannelのようなものを介して、bufferのやり取りを行えないか。
// loaderもsourceを取り込んだ形にしないほうがよいか?
// ondemand読み込みとconcurrent読み込みを両方扱えるようにしてもondemandはRcでくるむ必要があり、無駄になる。
// そのためondemandとconcurrentを分けて実装する。そうすれば単純な実装が可能になる。
pub trait IterativeParseSession {
    type Output;
    type Error;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<Self::Output>, Self::Error>>;
}

pub trait SequenceLoaderRuntime<L: SequenceLoader> {
    type Session: 'static + SequenceLoaderRuntimeSession<L>;

    fn spawn(&self, loader: L) -> Self::Session;
}

pub trait SequenceLoaderRuntimeSession<L: SequenceLoader> {
    type WaitForAppend<'a>: Future<Output = ()>
    where
        Self: 'a;

    fn notify_consumed(&self, len: L::Length);
    fn wait_for_append(&self) -> Self::WaitForAppend<'_>;
}
