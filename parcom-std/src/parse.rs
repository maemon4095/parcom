use std::marker::PhantomData;

use parcom_core::{Parse, ParseResult, Parser};

pub fn parser_for<P>() -> ParserFor<P> {
    ParserFor(PhantomData)
}

#[derive(Debug, Clone, Copy)]
pub struct ParserFor<P>(PhantomData<P>);

impl<T, P: Parse<T>> Parser<T> for ParserFor<P> {
    type Output = P;
    type Error = P::Error;
    type Fault = P::Fault;

    fn parse(&self, input: T) -> ParseResult<T, P, Self::Error, Self::Fault> {
        P::parse(input)
    }
}
