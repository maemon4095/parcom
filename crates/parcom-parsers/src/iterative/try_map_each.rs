use parcom_core::{
    IterativeParser, IterativeParserOnce, IterativeParserState, ParseError, ParseResult, Stream,
};
use parcom_util::{done, fail};
use std::marker::PhantomData;

pub struct TryMapEach<S, P, F>
where
    S: Stream,
    P: IterativeParserOnce<S>,
{
    parser: P,
    f: F,
    marker: PhantomData<S>,
}

impl<S, P, F> TryMapEach<S, P, F>
where
    S: Stream,
    P: IterativeParserOnce<S>,
{
    pub fn new<O, E>(parser: P, f: F) -> Self
    where
        F: Fn(P::Output) -> Result<O, E>,
        E: From<P::Error> + ParseError,
    {
        Self {
            parser,
            f,
            marker: PhantomData,
        }
    }
}

impl<S, P, O, E, F> IterativeParserOnce<S> for TryMapEach<S, P, F>
where
    S: Stream,
    P: IterativeParserOnce<S>,
    F: Fn(P::Output) -> Result<O, E>,
    E: From<P::Error> + ParseError,
{
    type Output = O;
    type Error = E;
    type StateOnce = IterationState<S, P::StateOnce, F>;

    fn start_once(self) -> Self::StateOnce {
        IterationState {
            state: self.parser.start_once(),
            f: self.f,
            marker: PhantomData,
        }
    }
}

impl<S, P, O, E, F> IterativeParser<S> for TryMapEach<S, P, F>
where
    S: Stream,
    P: IterativeParser<S>,
    F: Fn(P::Output) -> Result<O, E>,
    E: From<P::Error> + ParseError,
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

pub struct IterationState<S, P, F>
where
    S: Stream,
    P: IterativeParserState<S>,
{
    state: P,
    f: F,
    marker: PhantomData<S>,
}

impl<S, P, O, E, F> IterativeParserState<S> for IterationState<S, P, F>
where
    S: Stream,
    P: IterativeParserState<S>,
    F: Fn(P::Output) -> Result<O, E>,
    E: From<P::Error> + ParseError,
{
    type Output = O;
    type Error = E;

    async fn parse_next(&mut self, input: S) -> ParseResult<S, Option<Self::Output>, Self::Error> {
        match self.state.parse_next(input).await {
            Ok((Some(v), r)) => match (self.f)(v) {
                Ok(v) => done(Some(v), r),
                Err(e) => {
                    if e.should_terminate() {
                        fail(e, r)
                    } else {
                        done(None, r)
                    }
                }
            },
            Ok((None, r)) => done(None, r),
            Err(e) => Err(e.conv_fail()),
        }
    }
}
