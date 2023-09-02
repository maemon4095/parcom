use std::marker::PhantomData;

use crate::{ParseResult, Parser, RewindStream};

pub mod str;

pub fn func<S: RewindStream, O, E, F: Fn(S) -> Result<(O, S), (E, S)>>(
    func: F,
) -> Func<S, O, E, F> {
    Func {
        func,
        marker: PhantomData,
    }
}

pub struct Func<S, O, E, F: Fn(S) -> Result<(O, S), (E, S)>> {
    func: F,
    marker: PhantomData<(S, O, E)>,
}

impl<S, O, E, F: Fn(S) -> Result<(O, S), (E, S)>> Parser<S> for Func<S, O, E, F> {
    type Output = O;
    type Error = E;

    fn parse(&self, input: S) -> ParseResult<S, Self> {
        (self.func)(input)
    }
}
