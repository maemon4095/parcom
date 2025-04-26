use parcom_core::{
    IterativeParser, IterativeParserOnce, IterativeParserState, ParseResult::*, Stream,
};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct MapEach<S: Stream, P: IterativeParserOnce<S>, F> {
    parser: P,
    map: F,
    marker: PhantomData<S>,
}

impl<S: Stream, P: IterativeParserOnce<S>, F> MapEach<S, P, F> {
    pub fn new(parser: P, map: F) -> Self {
        Self {
            parser,
            map,
            marker: PhantomData,
        }
    }
}

impl<S: Stream, P: IterativeParserOnce<S>, T, F: Fn(P::Output) -> T> IterativeParserOnce<S>
    for MapEach<S, P, F>
{
    type Output = T;
    type Error = P::Error;
    type StateOnce = IterationState<S, P::StateOnce, T, F>;

    fn start_once(self) -> Self::StateOnce {
        IterationState {
            map: self.map,
            state: self.parser.start_once(),
            marker: PhantomData,
        }
    }
}

impl<S: Stream, P: IterativeParser<S>, T, F: Fn(P::Output) -> T> IterativeParser<S>
    for MapEach<S, P, F>
{
    type State<'a>
        = IterationState<S, P::State<'a>, T, &'a F>
    where
        Self: 'a;

    fn start(&self) -> Self::State<'_> {
        IterationState {
            map: &self.map,
            state: self.parser.start(),
            marker: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct IterationState<S: Stream, P: IterativeParserState<S>, T, F: Fn(P::Output) -> T> {
    map: F,
    state: P,
    marker: PhantomData<S>,
}

impl<S: Stream, P: IterativeParserState<S>, T, F: Fn(P::Output) -> T> IterativeParserState<S>
    for IterationState<S, P, T, F>
{
    type Output = T;
    type Error = P::Error;

    async fn parse_next(
        &mut self,
        input: S,
    ) -> parcom_core::ParseResult<S, Option<Self::Output>, Self::Error> {
        match self.state.parse_next(input).await {
            Done(v, r) => Done(v.map(&self.map), r),
            Fail(e, r) => Fail(e, r),
            StreamErr(e, r) => StreamErr(e, r),
        }
    }
}
