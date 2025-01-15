use parcom_core::{ParseError, ParseResult, Parser, ParserResult};
use std::marker::PhantomData;

pub struct Then<S, P, O, E, Fun>
where
    P: Parser<S>,
    Fun: Fn(ParserResult<S, P>) -> ParseResult<S, O, E>,
    E: ParseError,
{
    parser: P,
    f: Fun,
    marker: PhantomData<S>,
}

impl<S, P, O, E, Fun> Then<S, P, O, E, Fun>
where
    P: Parser<S>,
    E: ParseError,
    Fun: Fn(ParserResult<S, P>) -> ParseResult<S, O, E>,
{
    pub(super) fn new(parser: P, f: Fun) -> Self {
        Self {
            parser,
            f,
            marker: PhantomData,
        }
    }
}

impl<S, P, O, E, Fun> Parser<S> for Then<S, P, O, E, Fun>
where
    P: Parser<S>,
    E: ParseError,
    Fun: Fn(ParserResult<S, P>) -> ParseResult<S, O, E>,
{
    type Output = O;
    type Error = E;

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let r = self.parser.parse(input).await;
        (self.f)(r)
    }
}
