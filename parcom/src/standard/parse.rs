use std::marker::PhantomData;

use crate::{Parse, Parser};

pub fn parser_for<P>() -> ParserFor<P> {
    ParserFor(PhantomData)
}

#[derive(Debug, Clone, Copy)]
pub struct ParserFor<P>(PhantomData<P>);

impl<T, P: Parse<T>> Parser<T> for ParserFor<P> {
    type Output = P;
    type Error = P::Error;

    fn parse(&self, input: T) -> crate::ParseResult<T, P, Self::Error> {
        P::parse(input)
    }
}
