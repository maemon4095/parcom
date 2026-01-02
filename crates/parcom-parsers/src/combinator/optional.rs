use parcom_core::{
    IterativeParser, IterativeParserOnce, IterativeParserState, ParseError, Parser, ParserOnce,
    ParserResult, RewindSequence,
};
use parcom_util::{done, fail};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Optional<S: RewindSequence, P: ParserOnce<S>> {
    parser: P,
    marker: PhantomData<S>,
}

impl<T: RewindSequence, P: ParserOnce<T>> Optional<T, P> {
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            marker: PhantomData,
        }
    }
}
impl<S: RewindSequence, P: ParserOnce<S>> ParserOnce<S> for Optional<S, P> {
    type Output = Result<P::Output, P::Error>;
    type Error = P::Error;

    async fn parse_once(self, input: S) -> ParserResult<S, Self> {
        let anchor = input.anchor();
        match self.parser.parse_once(input).await {
            Ok((v, r)) => done(Ok(v), r),
            Err((e, r)) if !e.should_terminate() => done(Err(e), r.rewind(anchor).await),
            Err((e, r)) => fail(e, r),
        }
    }
}

impl<S: RewindSequence, P: Parser<S>> Parser<S> for Optional<S, P> {
    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let anchor = input.anchor();
        match self.parser.parse(input).await {
            Ok((v, r)) => done(Ok(v), r),
            Err((e, r)) if !e.should_terminate() => done(Err(e), r.rewind(anchor).await),
            Err((e, r)) => fail(e, r),
        }
    }
}

impl<S: RewindSequence, P: ParserOnce<S>> IterativeParserOnce<S> for Optional<S, P> {
    type Output = P::Output;
    type Error = P::Error;
    type StateOnce = IterationStateOnce<S, P>;

    fn parse_iterative_once(self) -> Self::StateOnce {
        IterationStateOnce { me: Some(self) }
    }
}

impl<S: RewindSequence, P: Parser<S>> IterativeParser<S> for Optional<S, P> {
    type State<'a>
        = IterationState<'a, S, P>
    where
        Self: 'a;
    fn parse_iterative(&self) -> Self::State<'_> {
        IterationState { me: Some(self) }
    }
}

#[derive(Debug)]
pub struct IterationStateOnce<S: RewindSequence, P: ParserOnce<S>> {
    me: Option<Optional<S, P>>,
}

impl<S: RewindSequence, P: ParserOnce<S>> IterativeParserState<S> for IterationStateOnce<S, P> {
    type Output = P::Output;
    type Error = P::Error;

    async fn parse_next(
        &mut self,
        input: S,
    ) -> parcom_core::ParseResult<S, Option<Self::Output>, Self::Error> {
        let Some(me) = self.me.take() else {
            return done(None, input);
        };

        let anchor = input.anchor();
        match me.parser.parse_once(input).await {
            Ok((v, r)) => done(Some(v), r),
            Err((e, r)) if e.should_terminate() => Err((e, r)),
            Err((_, r)) => done(None, r.rewind(anchor).await),
        }
    }
}

#[derive(Debug)]
pub struct IterationState<'a, S: RewindSequence, P: Parser<S>> {
    me: Option<&'a Optional<S, P>>,
}

impl<'a, S: RewindSequence, P: Parser<S>> IterativeParserState<S> for IterationState<'a, S, P> {
    type Output = P::Output;
    type Error = P::Error;

    async fn parse_next(
        &mut self,
        input: S,
    ) -> parcom_core::ParseResult<S, Option<Self::Output>, Self::Error> {
        let Some(me) = self.me.take() else {
            return done(None, input);
        };

        let anchor = input.anchor();
        match me.parser.parse(input).await {
            Ok((v, r)) => done(Some(v), r),
            Err((e, r)) if e.should_terminate() => Err((e, r)),
            Err((_, r)) => done(None, r.rewind(anchor).await),
        }
    }
}
