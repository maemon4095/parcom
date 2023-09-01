// TODO: implement cache with skip list
//  @: cache node
//
//  @ <--+
//  |    |
//  |    @ <---------+
//  |    |           |
//  |    |           @ <---- boundCache
//  |    |           |
//  |    |           |
// -+----+- stream --+----->

use std::{cell::RefCell, marker::PhantomData};

use crate::{packrat::smart_pointer::WeakRef, Location, Parser};

use super::BindStream;

struct Node<S: BindStream, P: Parser<S>> {
    location: S::Location,
    prev: S::Weak<Node<S, P>>,
    value: Result<(P::Output, usize), P::Error>,
}

struct BoundCache<S: BindStream, P: Parser<S>>
where
    P::Error: Clone,
    P::Output: Clone,
{
    pub(super) parser: P,
    pub(super) cache: RefCell<S::Weak<Node<S, P>>>,
    pub(super) marker: PhantomData<S>,
}

impl<S: BindStream, P: Parser<S>> Parser<S> for BoundCache<S, P>
where
    P::Error: Clone,
    P::Output: Clone,
{
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&self, input: S) -> crate::ParseResult<S, Self> {
        let location = input.location(0);
        let r = self.cache.borrow();
        match search::<S, P>(&*r, &location) {
            Some(result) => {
                return match result {
                    Ok((v, c)) => Ok((v, input.advance(c))),
                    Err(r) => Err((r, input)),
                }
            }
            None => (),
        }

        match self.parser.parse(input) {
            Ok((v, r)) => {
                let to = r.location(0);
                let delta = to.distance(&location);
            }
            Err(_) => todo!(),
        }

        todo!()
    }
}

fn search<S: BindStream, P: Parser<S>>(
    node: &S::Weak<Node<S, P>>,
    location: &S::Location,
) -> Option<Result<(P::Output, usize), P::Error>>
where
    P::Error: Clone,
    P::Output: Clone,
{
    let mut current = node.upgrade();
    while let Some(n) = current {
        let r = n.as_ref();
        if &r.location == location {
            return Some(r.value.clone());
        }

        current = r.prev.upgrade();
    }

    None
}
