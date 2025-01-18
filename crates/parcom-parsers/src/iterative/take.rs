use parcom_core::{
    IterativeParser, IterativeParserState,
    ParseResult::{self, *},
};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Take<S, P: IterativeParser<S>> {
    parser: P,
    count: usize,
    marker: PhantomData<S>,
}
impl<S, P: IterativeParser<S>> Take<S, P> {
    pub fn new(parser: P, count: usize) -> Self {
        Self {
            parser,
            count,
            marker: PhantomData,
        }
    }
}

impl<S, P: IterativeParser<S>> IterativeParser<S> for Take<S, P> {
    type Output = P::Output;
    type Error = Option<P::Error>;
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
pub struct IterationState<S, P: IterativeParserState<S>> {
    state: P,
    left: usize,
    marker: PhantomData<S>,
}

impl<S, P: IterativeParserState<S>> IterativeParserState<S> for IterationState<S, P> {
    type Output = P::Output;
    type Error = Option<P::Error>;

    async fn parse_next(
        &mut self,
        input: S,
    ) -> ParseResult<S, Result<Self::Output, Self::Error>, Self::Error> {
        if self.left == 0 {
            return Done(Err(None), input);
        }

        match self.state.parse_next(input).await {
            Done(v, r) => {
                self.left -= 1;
                Done(v.map_err(Some), r)
            }
            Fail(e, r) => Fail(Some(e), r),
        }
    }
}
