use crate::{Location, Parser};
use std::collections::BTreeMap;
use std::rc::Rc;
use std::{cell::RefCell, marker::PhantomData};

use super::BindStream;

pub struct BoundCached<S: BindStream, P: Parser<S>>
where
    S::Location: Clone,
    P::Error: Clone,
    P::Output: Clone,
{
    parser: P,
    cache: CacheServer<S, P>,
    marker: PhantomData<S>,
}

impl<S: BindStream, P: Parser<S>> BoundCached<S, P>
where
    S::Location: Clone,
    P::Error: Clone,
    P::Output: Clone,
{
    pub(super) fn new(parser: P) -> Self {
        Self {
            parser,
            cache: CacheServer::new(),
            marker: PhantomData,
        }
    }
}

impl<S: BindStream, P: Parser<S>> Parser<S> for BoundCached<S, P>
where
    S::Location: Clone,
    P::Error: Clone,
    P::Output: Clone,
{
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&self, input: S) -> crate::ParseResult<S, Self::Output, Self::Error> {
        let location = input.location(0);
        match self.cache.get(&location) {
            Some(Ok((o, c))) => return Ok((o, input.advance(c))),
            Some(Err(e)) => return Err((e, input)),
            None => (),
        }

        match self.parser.parse(input) {
            Ok((o, r)) => {
                let delta = r.location(0).distance(&location);
                self.cache.save(location, Ok((o.clone(), delta)));
                Ok((o, r))
            }
            Err((e, r)) => {
                self.cache.save(location, Err(e.clone()));
                Err((e, r))
            }
        }
    }
}

struct CacheServer<S: BindStream, P: Parser<S>>
where
    S::Location: Clone,
    P::Error: Clone,
    P::Output: Clone,
{
    server: Rc<RefCell<BTreeMap<S::Location, Result<(P::Output, usize), P::Error>>>>,
}

impl<S: BindStream, P: Parser<S>> CacheServer<S, P>
where
    S::Location: Clone,
    P::Error: Clone,
    P::Output: Clone,
{
    fn new() -> Self {
        Self {
            server: Rc::new(RefCell::new(BTreeMap::new())),
        }
    }

    fn save(
        &self,
        location: S::Location,
        result: Result<(P::Output, usize), P::Error>,
    ) -> CacheAnchor<S, P> {
        self.server.borrow_mut().insert(location.clone(), result);
        CacheAnchor {
            location,
            server: Self {
                server: self.server.clone(),
            },
        }
    }
    fn delete(&self, location: &S::Location) -> Option<Result<(P::Output, usize), P::Error>> {
        self.server.borrow_mut().remove(&location)
    }

    fn get(&self, location: &S::Location) -> Option<Result<(P::Output, usize), P::Error>> {
        let r = self.server.borrow();
        r.get(location).map(|e| e.clone())
    }
}

struct CacheAnchor<S: BindStream, P: Parser<S>>
where
    S::Location: Clone,
    P::Error: Clone,
    P::Output: Clone,
{
    location: S::Location,
    server: CacheServer<S, P>,
}

impl<S: BindStream, P: Parser<S>> Drop for CacheAnchor<S, P>
where
    S::Location: Clone,
    P::Error: Clone,
    P::Output: Clone,
{
    fn drop(&mut self) {
        self.server.delete(&self.location);
    }
}
