use parcom_core::{ParseResult, Parser, ParserResult};
use std::marker::PhantomData;

pub struct Then<S, P, O, E, F, Fun>
where
    P: Parser<S>,
    Fun: Fn(ParserResult<S, P>) -> ParseResult<S, O, E, F>,
{
    parser: P,
    f: Fun,
    marker: PhantomData<(S, O, E, F)>,
}

impl<S, P, O, E, F, Fun> Then<S, P, O, E, F, Fun>
where
    P: Parser<S>,
    Fun: Fn(ParserResult<S, P>) -> ParseResult<S, O, E, F>,
{
    pub(super) fn new(parser: P, f: Fun) -> Self {
        Self {
            parser,
            f,
            marker: PhantomData,
        }
    }
}

impl<S, P, O, E, F, Fun> Parser<S> for Then<S, P, O, E, F, Fun>
where
    P: Parser<S>,
    Fun: Fn(ParserResult<S, P>) -> ParseResult<S, O, E, F>,
{
    type Output = O;
    type Error = E;
    type Fault = F;

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let r = self.parser.parse(input).await;
        (self.f)(r)
    }
}
