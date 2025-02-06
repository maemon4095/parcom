use parcom_core::{
    IterativeParser, IterativeParserOnce, IterativeParserState, ParseError,
    ParseResult::{self, *},
    Parser, ParserOnce, RewindStream,
};
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
            Done(None, r) => {
                return Done((collection, None), r.rewind(anchor).await);
            }
            Done(v, r) => {
                collection.extend(v);
                rest = r;
            }
            Fail(e, r) if e.should_terminate() => return Fail(e, r),
            Fail(e, r) => {
                return Done((collection, Some(e)), r.rewind(anchor).await);
            }
        }
    }
}
