use std::marker::PhantomData;

use crate::Parser;
pub struct AsRef<'a, S, P: Parser<S>> {
    pub(super) parser: &'a P,
    pub(super) marker: PhantomData<S>,
}

impl<'a, S, P: Parser<S>> Parser<S> for AsRef<'a, S, P> {
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&self, input: S) -> parcom_core::ParseResult<S, Self::Output, Self::Error> {
        self.parser.parse(input)
    }
}
