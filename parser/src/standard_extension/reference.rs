use std::marker::PhantomData;

use crate::Parser;

#[derive(Clone, Copy)]
pub struct Ref<'a, S, P: Parser<S>> {
    pub(super) parser: &'a P,
    pub(super) marker: PhantomData<S>,
}

impl<'a, S, P: Parser<S>> Parser<S> for Ref<'a, S, P> {
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&self, input: S) -> crate::ParseResult<S, Self> {
        self.parser.parse(input)
    }
}
