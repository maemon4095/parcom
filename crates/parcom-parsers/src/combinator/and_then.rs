use std::marker::PhantomData;

use parcom_core::{ParseError, Parser, ParserOnce, ParserResult, Sequence};
use parcom_util::{done, fail, ParseResultExt};

#[derive(Debug)]
pub struct AndThen<S, P, O, E, F>
where
    S: Sequence,
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
    S: Sequence,
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
    S: Sequence,
    P: ParserOnce<S>,
    F: Fn(P::Output) -> Result<O, E>,
    E: ParseError + From<P::Error>,
{
    type Output = O;
    type Error = E;

    async fn parse_once(self, input: S) -> ParserResult<S, Self> {
        let (v, r) = self.parser.parse_once(input).await.conv_fail()?;
        match (self.map)(v) {
            Ok(v) => done(v, r),
            Err(e) => fail(e, r),
        }
    }
}

impl<S, P, O, E, F> Parser<S> for AndThen<S, P, O, E, F>
where
    S: Sequence,
    P: Parser<S>,
    F: Fn(P::Output) -> Result<O, E>,
    E: ParseError + From<P::Error>,
{
    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let (v, r) = self.parser.parse(input).await.conv_fail()?;
        match (self.map)(v) {
            Ok(v) => done(v, r),
            Err(e) => fail(e, r),
        }
    }
}
