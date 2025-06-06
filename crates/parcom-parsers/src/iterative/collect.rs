use parcom_core::{
    Error, IterativeParser, IterativeParserOnce, IterativeParserState, ParseResult, Parser,
    ParserOnce, RewindStream,
};
use parcom_util::{done, ResultExt};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Collect<S: RewindStream, P: IterativeParserOnce<S>, C: Extend<P::Output> + Default> {
    parser: P,
    marker: PhantomData<(S, C)>,
}

impl<S: RewindStream, P: IterativeParserOnce<S>, C: Extend<P::Output> + Default> Collect<S, P, C> {
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            marker: PhantomData,
        }
    }
}
impl<S: RewindStream, P: IterativeParserOnce<S>, C: Extend<P::Output> + Default> ParserOnce<S>
    for Collect<S, P, C>
{
    type Output = (C, Option<P::Error>);
    type Error = P::Error;

    async fn parse_once(self, input: S) -> parcom_core::ParserResult<S, Self> {
        parse(self.parser.start_once(), input).await
    }
}

impl<S: RewindStream, P: IterativeParser<S>, C: Extend<P::Output> + Default> Parser<S>
    for Collect<S, P, C>
{
    async fn parse(&self, input: S) -> parcom_core::ParseResult<S, Self::Output, Self::Error> {
        parse(self.parser.start(), input).await
    }
}

async fn parse<S: RewindStream, P: IterativeParserState<S>, C: Extend<P::Output> + Default>(
    mut state: P,
    input: S,
) -> ParseResult<S, (C, Option<P::Error>), P::Error> {
    let mut collection = C::default();
    let mut rest = input;
    loop {
        let anchor = rest.anchor();

        match state.parse_next(rest).await {
            Ok((None, r)) => {
                return done((collection, None), r.rewind(anchor).await.stream_err()?);
            }
            Ok((Some(v), r)) => {
                collection.extend(std::iter::once(v));
                rest = r;
            }
            Err(Error::Fail(e, r)) => {
                return done((collection, Some(e)), r.rewind(anchor).await.stream_err()?);
            }
            Err(e) => return Err(e),
        }
    }
}
