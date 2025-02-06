use std::marker::PhantomData;

use parcom_core::{
    ParseError,
    ParseResult::{self, *},
    Parser, ParserOnce,
};

pub struct AndThen<S, P, O, E, F>
where
    P: ParserOnce<S>,
    F: Fn(P::Output) -> Result<O, E>,
    E: ParseError + From<P::Error>,
{
    parser: P,
    map: F,
    marker: PhantomData<(S, O, E)>,
}
impl<S, P, O, E, F> AndThen<S, P, O, E, F>
where
    P: ParserOnce<S>,
    F: Fn(P::Output) -> Result<O, E>,
    E: ParseError + From<P::Error>,
{
    pub fn new(parser: P, map: F) -> Self {
        Self {
            parser,
            map,
            marker: PhantomData,
        }
    }
}

impl<S, P, O, E, F> ParserOnce<S> for AndThen<S, P, O, E, F>
where
    P: ParserOnce<S>,
    F: Fn(P::Output) -> Result<O, E>,
    E: ParseError + From<P::Error>,
{
    type Output = O;
    type Error = E;

    async fn parse_once(self, input: S) -> parcom_core::ParserResult<S, Self> {
        match self.parser.parse_once(input).await {
            Done(v, r) => match (self.map)(v) {
                Ok(v) => Done(v, r),
                Err(e) => Fail(e, r.into()),
            },
            Fail(e, r) => Fail(e.into(), r),
        }
    }
}

impl<S, P, O, E, F> Parser<S> for AndThen<S, P, O, E, F>
where
    P: Parser<S>,
    F: Fn(P::Output) -> Result<O, E>,
    E: ParseError + From<P::Error>,
{
    async fn parse(&self, input: S) -> ParseResult<S, Self::Output, Self::Error> {
        match self.parser.parse(input).await {
            Done(v, r) => match (self.map)(v) {
                Ok(v) => Done(v, r),
                Err(e) => Fail(e, r.into()),
            },
            Fail(e, r) => Fail(e.into(), r),
        }
    }
}
