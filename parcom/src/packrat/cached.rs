use std::collections::BTreeMap;
use std::{cell::RefCell, marker::PhantomData};

use crate::{Location, ParseResult, ParseStream, Parser};

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

    fn parse(&self, input: S) -> ParseResult<S, Self> {
        let location = input.location(0);
        match self.server.borrow().get(&location) {
            Some(result) => {
                return match result {
                    Ok((v, c)) => Ok((v.clone(), input.advance(*c))),
                    Err(e) => Err((e.clone(), input)),
                }
            }
            None => (),
        }

        match self.parser.parse(input) {
            Ok((v, r)) => {
                let tail = r.location(0);
                let distance = tail.distance(&location);
                self.server
                    .borrow_mut()
                    .insert(location, Ok((v.clone(), distance)));
                Ok((v, r))
            }
            Err((e, r)) => {
                self.server.borrow_mut().insert(location, Err(e.clone()));
                Err((e, r))
            }
        }
    }
}
