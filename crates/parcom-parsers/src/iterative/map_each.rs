use parcom_core::{IterativeParser, IterativeParserState, ParseResult::*};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct MapEach<S, P: IterativeParser<S>, T, F: Fn(P::Output) -> T> {
    parser: P,
    map: F,
    marker: PhantomData<S>,
}

impl<S, P: IterativeParser<S>, T, F: Fn(P::Output) -> T> MapEach<S, P, T, F> {
    pub fn new(parser: P, map: F) -> Self {
        Self {
            parser,
            map,
            marker: PhantomData,
        }
    }
}

impl<S, P: IterativeParser<S>, T, F: Fn(P::Output) -> T> IterativeParser<S>
    for MapEach<S, P, T, F>
{
    type Output = T;
    type Error = P::Error;
    type PrerequisiteError = P::PrerequisiteError;
    type State<'a>
        = IterationState<'a, S, P, T, F>
    where
        Self: 'a;

    fn start(&self) -> Self::State<'_> {
        IterationState {
            map: &self.map,
            state: self.parser.start(),
        }
    }
}

#[derive(Debug)]
pub struct IterationState<'a, S, P: 'a + IterativeParser<S>, T, F: Fn(P::Output) -> T> {
    map: &'a F,
    state: P::State<'a>,
}

impl<'a, S, P: 'a + IterativeParser<S>, T, F: Fn(P::Output) -> T> IterativeParserState<S>
    for IterationState<'a, S, P, T, F>
{
    type Output = T;
    type Error = P::Error;
    type PrerequisiteError = P::PrerequisiteError;

    fn prerequisite_error(&self) -> Option<Self::PrerequisiteError> {
        self.state.prerequisite_error()
    }

    async fn parse_next(
        &mut self,
        input: S,
    ) -> parcom_core::ParseResult<S, Result<Self::Output, Self::Error>, Self::Error> {
        match self.state.parse_next(input).await {
            Done(v, r) => Done(v.map(self.map), r),
            Fail(e, r) => Fail(e, r),
        }
    }
}
