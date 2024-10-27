use std::future::Future;

use crate::ParseResult;

pub trait Parser<S> {
    type Output;
    type Error;
    type Fault;

    fn parse(
        &self,
        input: S,
    ) -> impl Future<Output = ParseResult<S, Self::Output, Self::Error, Self::Fault>>;
}

impl<S, O, E, F, Fut, T: Fn(S) -> Fut> Parser<S> for T
where
    Fut: Future<Output = ParseResult<S, O, E, F>>,
{
    type Output = O;
    type Error = E;
    type Fault = F;

    fn parse(&self, input: S) -> impl Future<Output = ParseResult<S, O, E, F>> {
        self(input)
    }
}
