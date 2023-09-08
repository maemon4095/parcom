use std::marker::PhantomData;

use crate::{ParseResult::*, Parser, RewindStream};

pub fn iterate<S: RewindStream, P: Parser<S>>(input: S, parser: P) -> Iter<S, P> {
    Iter {
        rest: input,
        last_err: None,
        parser,
        marker: PhantomData,
    }
}

pub struct Iter<S: RewindStream, P: Parser<S>> {
    rest: S,
    last_err: Option<P::Error>,
    parser: P,
    marker: PhantomData<S>,
}

impl<S: RewindStream, P: Parser<S>> Iter<S, P> {
    pub fn rest(&self) -> &S {
        &self.rest
    }

    pub fn last_err(&self) -> Option<&P::Error> {
        self.last_err.as_ref()
    }

    pub fn deconstruct(self) -> (S, P, Option<P::Error>) {
        (self.rest, self.parser, self.last_err)
    }

    pub fn next(&mut self) -> Result<P::Output, P::Error> {
        unsafe {
            let ptr: *mut S = &mut self.rest;
            let rest = ptr.read();
            let anchor = rest.anchor();
            match self.parser.parse(rest) {
                Done(item, rest) => {
                    ptr.write(rest);
                    Ok(item)
                }
                Fail(e, rest) => {
                    ptr.write(rest.rewind(anchor));
                    Err(e)
                }
            }
        }
    }
}

impl<S: RewindStream, P: Parser<S>> Iterator for Iter<S, P> {
    type Item = Result<P::Output, P::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next())
    }
}
