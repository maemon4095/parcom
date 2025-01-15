use std::future::Future;

use crate::{ParseError, ParseResult};

pub trait Parser<S> {
    type Output;
    type Error: ParseError;

    fn parse(&self, input: S) -> impl Future<Output = ParseResult<S, Self::Output, Self::Error>>;
}

impl<S, O, E: ParseError, Fut, T: Fn(S) -> Fut> Parser<S> for T
where
    Fut: Future<Output = ParseResult<S, O, E>>,
{
    type Output = O;
    type Error = E;

    fn parse(&self, input: S) -> impl Future<Output = ParseResult<S, O, E>> {
        self(input)
    }
}
