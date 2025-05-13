use std::marker::PhantomData;

use parcom_core::{
    IterativeParser, IterativeParserOnce, IterativeParserState, ParseResult, Stream,
};

pub struct MapWhile<S: Stream, P: IterativeParserOnce<S>, F> {
    parser: P,
    f: F,
    marker: PhantomData<S>,
}

impl<S: Stream, P: IterativeParserOnce<S>, F> MapWhile<S, P, F> {
    pub fn new(parser: P, f: F) -> Self {
        Self {
            parser,
            f,
            marker: PhantomData,
        }
    }
}

impl<S, P, O, F> IterativeParserOnce<S> for MapWhile<S, P, F>
where
    S: Stream,
    P: IterativeParserOnce<S>,
    F: Fn(P::Output) -> Option<O>,
{
    type Output = O;
    type Error = P::Error;
    type StateOnce = IterationState<S, P::StateOnce, F>;

    fn start_once(self) -> Self::StateOnce {
        IterationState {
            state: self.parser.start_once(),
            f: self.f,
            marker: PhantomData,
        }
    }
}

impl<S, P, O, F> IterativeParser<S> for MapWhile<S, P, F>
where
    S: Stream,
    P: IterativeParser<S>,
    F: Fn(P::Output) -> Option<O>,
{
    type State<'a>
        = IterationState<S, P::State<'a>, &'a F>
    where
        Self: 'a;

    fn start(&self) -> Self::State<'_> {
        IterationState {
            state: self.parser.start(),
            f: &self.f,
            marker: PhantomData,
        }
    }
}

pub struct IterationState<S: Stream, P: IterativeParserState<S>, F> {
    state: P,
    f: F,
    marker: PhantomData<S>,
}

impl<S, P, O, F> IterativeParserState<S> for IterationState<S, P, F>
where
    S: Stream,
    P: IterativeParserState<S>,
    F: Fn(P::Output) -> Option<O>,
{
    type Output = O;
    type Error = P::Error;

    async fn parse_next(&mut self, input: S) -> ParseResult<S, Option<Self::Output>, Self::Error> {
        self.state
            .parse_next(input)
            .await
            .map(|(v, r)| (v.and_then(&self.f), r))
    }
}
