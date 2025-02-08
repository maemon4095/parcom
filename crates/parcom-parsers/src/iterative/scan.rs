use std::marker::PhantomData;

use parcom_core::{IterativeParser, IterativeParserOnce, IterativeParserState, ParseResult};

pub struct Scan<S, P, St, F>
where
    P: IterativeParserOnce<S>,
{
    parser: P,
    f: F,
    initial_state: St,
    marker: PhantomData<S>,
}

impl<S, P, St, F> Scan<S, P, St, F>
where
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
    P: IterativeParserOnce<S>,
    F: Fn(&mut St, P::Output) -> O,
{
    type Output = O;
    type Error = P::Error;
    type StateOnce = IterationState<S, P::StateOnce, St, F>;

    fn start_once(self) -> Self::StateOnce {
        IterationState {
            parser_state: self.parser.start_once(),
            f: self.f,
            state: self.initial_state,
            marker: PhantomData,
        }
    }
}

impl<S, P, St, F, O> IterativeParser<S> for Scan<S, P, St, F>
where
    P: IterativeParser<S>,
    F: Fn(&mut St, P::Output) -> O,
    St: Clone,
{
    type State<'a>
        = IterationState<S, P::State<'a>, St, &'a F>
    where
        Self: 'a;

    fn start(&self) -> Self::State<'_> {
        IterationState {
            parser_state: self.parser.start(),
            f: &self.f,
            state: self.initial_state.clone(),
            marker: PhantomData,
        }
    }
}

pub struct IterationState<S, P, St, F>
where
    P: IterativeParserState<S>,
{
    parser_state: P,
    f: F,
    state: St,
    marker: PhantomData<(S,)>,
}

impl<S, P, St, F, O> IterativeParserState<S> for IterationState<S, P, St, F>
where
    F: Fn(&mut St, P::Output) -> O,
    P: IterativeParserState<S>,
{
    type Output = O;
    type Error = P::Error;

    async fn parse_next(&mut self, input: S) -> ParseResult<S, Option<Self::Output>, Self::Error> {
        match self.parser_state.parse_next(input).await {
            ParseResult::Done(Some(v), r) => {
                let o = (self.f)(&mut self.state, v);
                ParseResult::Done(Some(o), r)
            }
            ParseResult::Done(None, r) => ParseResult::Done(None, r),
            ParseResult::Fail(e, r) => ParseResult::Fail(e, r),
        }
    }
}
