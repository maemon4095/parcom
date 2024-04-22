use crate::{ParseResult, ParserResult};

pub trait Parser<S> {
    type Output;
    type Error;
    type Fault;

    fn parse(&self, input: S) -> ParserResult<S, Self>;
}

impl<S, O, E, F, T: Fn(S) -> ParseResult<S, O, E, F>> Parser<S> for T {
    type Output = O;
    type Error = E;
    type Fault = F;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
        self(input)
    }
}

impl<S, O, E, F> Parser<S> for Box<dyn Parser<S, Output = O, Error = E, Fault = F>> {
    type Output = O;
    type Error = E;
    type Fault = F;

    fn parse(&self, input: S) -> ParseResult<S, O, E, F> {
        self.as_ref().parse(input)
    }
}
