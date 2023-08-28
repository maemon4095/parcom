pub mod str;
use crate::Parser;

impl<S, O, E, F: Fn(S) -> Result<(O, S), (E, S)>> Parser<S> for F {
    type Output = O;
    type Error = E;

    fn parse(&self, input: S) -> Result<(Self::Output, S), (Self::Error, S)> {
        self(input)
    }
}

impl<S, O, E> Parser<S> for Box<dyn Parser<S, Output = O, Error = E>> {
    type Output = O;
    type Error = E;

    fn parse(&self, input: S) -> Result<(Self::Output, S), (Self::Error, S)> {
        self.as_ref().parse(input)
    }
}
