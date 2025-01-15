use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum ParseResult<S, O, E: ParseError> {
    Done(O, S),
    Fail(E, UnknownLocation<S>),
}

use ParseResult::*;

use crate::{ParseError, UnknownLocation};

impl<S, O, E: ParseError> ParseResult<S, O, E> {
    pub fn map<T>(self, f: impl FnOnce(O) -> T) -> ParseResult<S, T, E> {
        match self {
            Done(v, r) => Done(f(v), r),
            Fail(e, r) => Fail(e, r),
        }
    }

    pub fn map_err<T: ParseError>(self, f: impl FnOnce(E) -> T) -> ParseResult<S, O, T> {
        match self {
            Done(v, r) => Done(v, r),
            Fail(e, r) => Fail(f(e), r),
        }
    }

    pub fn unwrap(self) -> (O, S)
    where
        E: Debug,
    {
        match self {
            Done(v, r) => (v, r),
            Fail(e, _) => panic!(
                "called ParseResult::unwrap on an Fail value; Error: {:?}.",
                e
            ),
        }
    }
    pub fn as_ref(&self) -> ParseResult<&S, &O, &E> {
        match self {
            Done(v, r) => Done(v, r),
            Fail(e, r) => Fail(e, r.as_ref()),
        }
    }

    pub fn as_result(&self) -> Result<&O, &E> {
        match self {
            Done(v, _) => Ok(v),
            Fail(e, _) => Err(e),
        }
    }
}
