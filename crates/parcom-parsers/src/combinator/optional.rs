use parcom_core::{
    IterativeParser, IterativeParserState, ParseError, ParseResult::*, Parser, ParserResult,
    RewindStream,
};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Optional<S: RewindStream, P: Parser<S>> {
    parser: P,
    marker: PhantomData<S>,
}

impl<T: RewindStream, P: Parser<T>> Optional<T, P> {
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            marker: PhantomData,
        }
    }
}

impl<S: RewindStream, P: Parser<S>> Parser<S> for Optional<S, P> {
    type Output = Result<P::Output, P::Error>;
    type Error = P::Error;

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let anchor = input.anchor();
        match self.parser.parse(input).await {
            Done(v, r) => Done(Ok(v), r),
            Fail(e, r) if !e.should_terminate() => Done(Err(e), r.rewind(anchor).await),
            Fail(e, r) => Fail(e, r),
        }
    }
}

impl<S: RewindStream, P: Parser<S>> IterativeParser<S> for Optional<S, P> {
    type Output = P::Output;
    type Error = P::Error;
    type State<'a>
        = IterationState<'a, S, P>
    where
        Self: 'a;

    fn start(&self) -> Self::State<'_> {
        IterationState { me: self }
    }
}

#[derive(Debug)]
pub struct IterationState<'a, S: RewindStream, P: Parser<S>> {
    me: &'a Optional<S, P>,
}

impl<'a, S: RewindStream, P: Parser<S>> IterativeParserState<S> for IterationState<'a, S, P> {
    type Output = P::Output;
    type Error = P::Error;

    async fn parse_next(
        &mut self,
        input: S,
    ) -> parcom_core::ParseResult<S, Result<Self::Output, Self::Error>, Self::Error> {
        self.me.parse(input).await
    }
}
