use parcom_core::{Parser, ParserOnce, ParserResult};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Ref<'a, S, P: ?Sized + Parser<S>> {
    parser: &'a P,
    marker: PhantomData<S>,
}
impl<'a, S, P: ?Sized + Parser<S>> Ref<'a, S, P> {
    pub fn new(parser: &'a P) -> Self {
        Self {
            parser,
            marker: PhantomData,
        }
    }
}

impl<'a, S, P: Parser<S>> Clone for Ref<'a, S, P> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser,
            marker: PhantomData,
        }
    }
}

impl<'a, S, P: Parser<S>> Copy for Ref<'a, S, P> {}

impl<'a, S, P: Parser<S>> ParserOnce<S> for Ref<'a, S, P> {
    type Output = P::Output;
    type Error = P::Error;

    async fn parse_once(self, input: S) -> ParserResult<S, Self> {
        self.parse(input).await
    }
}
impl<'a, S, P: Parser<S>> Parser<S> for Ref<'a, S, P> {
    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        self.parser.parse(input).await
    }
}
