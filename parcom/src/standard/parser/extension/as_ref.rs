use std::marker::PhantomData;

use crate::Parser;

pub struct AsRef<'a, S, P: ?Sized + Parser<S>> {
    pub(super) parser: &'a P,
    pub(super) marker: PhantomData<S>,
}

impl<'a, S, P: Parser<S>> Clone for AsRef<'a, S, P> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser,
            marker: PhantomData,
        }
    }
}

impl<'a, S, P: Parser<S>> Copy for AsRef<'a, S, P> {}

impl<'a, S, P: Parser<S>> Parser<S> for AsRef<'a, S, P> {
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&self, input: S) -> parcom_core::ParseResult<S, Self::Output, Self::Error> {
        self.parser.parse(input)
    }
}
