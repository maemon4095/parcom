use parcom_core::{
    IterativeParser, IterativeParserOnce, IterativeParserState, ParseResult, Sequence,
};
use parcom_util::done;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Take<S: Sequence, P: IterativeParser<S>> {
    parser: P,
    count: usize,
    marker: PhantomData<S>,
}
impl<S: Sequence, P: IterativeParser<S>> Take<S, P> {
    pub fn new(parser: P, count: usize) -> Self {
        Self {
            parser,
            count,
            marker: PhantomData,
        }
    }
}

impl<S: Sequence, P: IterativeParser<S>> IterativeParserOnce<S> for Take<S, P> {
    type Output = P::Output;
    type Error = P::Error;
    type StateOnce = IterationState<S, P::StateOnce>;

    fn start_once(self) -> Self::StateOnce {
        IterationState {
            state: self.parser.start_once(),
            left: self.count,
            marker: PhantomData,
        }
    }
}

impl<S: Sequence, P: IterativeParser<S>> IterativeParser<S> for Take<S, P> {
    type State<'a>
        = IterationState<S, P::State<'a>>
    where
        Self: 'a;

    fn start(&self) -> Self::State<'_> {
        IterationState {
            state: self.parser.start(),
            left: self.count,
            marker: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct IterationState<S: Sequence, P: IterativeParserState<S>> {
    state: P,
    left: usize,
    marker: PhantomData<S>,
}

impl<S: Sequence, P: IterativeParserState<S>> IterativeParserState<S> for IterationState<S, P> {
    type Output = P::Output;
    type Error = P::Error;

    async fn parse_next(&mut self, input: S) -> ParseResult<S, Option<Self::Output>, Self::Error> {
        if self.left == 0 {
            return done(None, input);
        }

        match self.state.parse_next(input).await? {
            (Some(v), r) => {
                self.left -= 1;
                done(Some(v), r)
            }
            (None, r) => done(None, r),
        }
    }
}
