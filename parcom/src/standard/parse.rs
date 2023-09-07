use std::marker::PhantomData;

use crate::{Parse, Parser};

pub fn parser_for<P>() -> ParserFor<P> {
    ParserFor(PhantomData)
}

pub struct ParserFor<P>(pub(super) PhantomData<P>);

impl<T, P: Parse<T>> Parser<T> for ParserFor<P> {
    type Output = P;
    type Error = P::Error;

    fn parse(&self, input: T) -> crate::ParseResult<T, P, Self::Error> {
        P::parse(input)
    }
}
