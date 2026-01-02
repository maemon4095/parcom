use std::marker::PhantomData;

use parcom_core::{
    IterativeParser, IterativeParserOnce, IterativeParserState, ParseResult, Sequence,
};
use parcom_util::done;

pub struct Scan<S, P, St, F>
where
    S: Sequence,
    P: IterativeParserOnce<S>,
{
    parser: P,
    f: F,
    initial_state: St,
    marker: PhantomData<S>,
}

impl<S, P, St, F> Scan<S, P, St, F>
where
    S: Sequence,
    P: IterativeParserOnce<S>,
{
    pub fn new(parser: P, initial_state: St, f: F) -> Self {
        Self {
            parser,
            f,
            initial_state,
            marker: PhantomData,
        }
    }
}

impl<S, P, St, F, O> IterativeParserOnce<S> for Scan<S, P, St, F>
where
    S: Sequence,
    P: IterativeParserOnce<S>,
    F: Fn(&mut St, P::Output) -> O,
{
    type Output = O;
    type Error = P::Error;
    type StateOnce = IterationState<S, P::StateOnce, St, F>;

    fn parse_iterative_once(self) -> Self::StateOnce {
        IterationState {
            parser_state: self.parser.parse_iterative_once(),
            f: self.f,
            state: self.initial_state,
            marker: PhantomData,
        }
    }
}

impl<S, P, St, F, O> IterativeParser<S> for Scan<S, P, St, F>
where
    S: Sequence,
    P: IterativeParser<S>,
    F: Fn(&mut St, P::Output) -> O,
    St: Clone,
{
    type State<'a>
        = IterationState<S, P::State<'a>, St, &'a F>
    where
        Self: 'a;

    fn parse_iterative(&self) -> Self::State<'_> {
        IterationState {
            parser_state: self.parser.parse_iterative(),
            f: &self.f,
            state: self.initial_state.clone(),
            marker: PhantomData,
        }
    }
}

pub struct IterationState<S, P, St, F>
where
    S: Sequence,
    P: IterativeParserState<S>,
{
    parser_state: P,
    f: F,
    state: St,
    marker: PhantomData<(S,)>,
}

impl<S, P, St, F, O> IterativeParserState<S> for IterationState<S, P, St, F>
where
    S: Sequence,
    F: Fn(&mut St, P::Output) -> O,
    P: IterativeParserState<S>,
{
    type Output = O;
    type Error = P::Error;

    async fn parse_next(&mut self, input: S) -> ParseResult<S, Option<Self::Output>, Self::Error> {
        match self.parser_state.parse_next(input).await? {
            (Some(v), r) => {
                let o = (self.f)(&mut self.state, v);
                done(Some(o), r)
            }
            (None, r) => done(None, r),
        }
    }
}
