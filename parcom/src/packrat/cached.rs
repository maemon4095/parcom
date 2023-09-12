use std::collections::BTreeMap;
use std::{cell::RefCell, marker::PhantomData};

use crate::{
    Location,
    ParseResult::{self, *},
    ParseStream, Parser, ParserResult,
};

pub struct Cached<S: ParseStream, P: Parser<S>>
where
    P::Error: Clone,
    P::Output: Clone,
{
    parser: P,
    server: RefCell<BTreeMap<S::Location, Result<(P::Output, usize), P::Error>>>,
    marker: PhantomData<S>,
}

impl<S: ParseStream, P: Parser<S>> Cached<S, P>
where
    P::Error: Clone,
    P::Output: Clone,
{
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            server: RefCell::new(BTreeMap::new()),
            marker: PhantomData,
        }
    }
}

impl<S: ParseStream, P: Parser<S>> Parser<S> for Cached<S, P>
where
    P::Error: Clone,
    P::Output: Clone,
{
    type Output = P::Output;
    type Error = P::Error;
    type Fault = P::Fault;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
        let location = input.location(0);
        match self.server.borrow().get(&location) {
            Some(result) => {
                return match result {
                    Ok((v, c)) => Done(v.clone(), input.advance(*c)),
                    Err(e) => Fail(e.clone(), input),
                }
            }
            None => (),
        }

        match self.parser.parse(input) {
            Done(v, r) => {
                let tail = r.location(0);
                let delta = tail.delta(&location);
                self.server
                    .borrow_mut()
                    .insert(location, Ok((v.clone(), delta.abs())));
                Done(v, r)
            }
            Fail(e, r) => {
                self.server.borrow_mut().insert(location, Err(e.clone()));
                Fail(e, r)
            }
            Fatal(e) => Fatal(e),
        }
    }
}
