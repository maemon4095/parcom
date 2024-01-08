use std::marker::PhantomData;

use crate::{ParseResult::*, Parser, RewindStream};

pub enum Result<O, E, F> {
    Ok(O),
    Err(E),
    Fault(F),
}

pub fn iterate<S: RewindStream, P: Parser<S>>(input: S, parser: P) -> Iter<S, P> {
    Iter {
        rest: Some(input),
        parser,
        marker: PhantomData,
    }
}

pub struct Iter<S: RewindStream, P: Parser<S>> {
    rest: Option<S>,
    parser: P,
    marker: PhantomData<S>,
}

impl<S: RewindStream, P: Parser<S>> Iter<S, P> {
    pub fn rest(&self) -> Option<&S> {
        self.rest.as_ref()
    }

    pub fn deconstruct(self) -> (Option<S>, P) {
        (self.rest, self.parser)
    }

    pub fn next(&mut self) -> Option<Result<P::Output, P::Error, P::Fault>> {
        use Result::*;
        let Some(rest) = self.rest.take() else {
            return None;
        };
        let anchor = rest.anchor();
        match self.parser.parse(rest) {
            Done(item, rest) => {
                self.rest.replace(rest);
                Some(Ok(item))
            }
            Fail(e, rest) => {
                self.rest.replace(rest.rewind(anchor));
                Some(Err(e))
            }
            Fatal(e) => Some(Fault(e)),
        }
    }
}

impl<S: RewindStream, P: Parser<S>> Iterator for Iter<S, P> {
    type Item = Result<P::Output, P::Error, P::Fault>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}
