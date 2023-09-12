use std::fmt::Debug;

#[derive(Debug, Clone, Copy)]
pub enum ParseResult<S, O, E, F = crate::Never> {
    Done(O, S),
    Fail(E, S),
    Fatal(F),
}

use crate::Result::{self, *};
use ParseResult::*;

impl<S, O, E, F> ParseResult<S, O, E, F> {
    pub fn map<T>(self, f: impl FnOnce(O) -> T) -> ParseResult<S, T, E, F> {
        match self {
            Done(v, r) => Done(f(v), r),
            Fail(e, r) => Fail(e, r),
            Fatal(e) => Fatal(e),
        }
    }

    pub fn map_err<T>(self, f: impl FnOnce(E) -> T) -> ParseResult<S, O, T, F> {
        match self {
            Done(v, r) => Done(v, r),
            Fail(e, r) => Fail(f(e), r),
            Fatal(e) => Fatal(e),
        }
    }

    pub fn map_fault<T>(self, f: impl FnOnce(F) -> T) -> ParseResult<S, O, E, T> {
        match self {
            Done(v, r) => Done(v, r),
            Fail(e, r) => Fail(e, r),
            Fatal(e) => Fatal(f(e)),
        }
    }

    pub fn and_then<T>(self, f: impl FnOnce(O) -> Result<T, E, F>) -> ParseResult<S, T, E, F> {
        match self {
            Done(v, r) => match f(v) {
                Ok(v) => Done(v, r),
                Err(e) => Fail(e, r),
                Fault(e) => Fatal(e),
            },
            Fail(e, r) => Fail(e, r),
            Fatal(e) => Fatal(e),
        }
    }

    pub fn or_else<T>(self, f: impl FnOnce(E) -> Result<O, T, F>) -> ParseResult<S, O, T, F> {
        match self {
            Done(v, r) => Done(v, r),
            Fail(e, r) => match f(e) {
                Ok(v) => Done(v, r),
                Err(e) => Fail(e, r),
                Fault(e) => Fatal(e),
            },
            Fatal(e) => Fatal(e),
        }
    }

    pub fn ok(self) -> Option<O> {
        match self {
            Done(v, _) => Some(v),
            _ => None,
        }
    }

    pub fn err(self) -> Option<E> {
        match self {
            Fail(e, _) => Some(e),
            _ => None,
        }
    }

    pub fn fault(self) -> Option<F> {
        match self {
            Fatal(e) => Some(e),
            _ => None,
        }
    }

    pub fn unwrap(self) -> (O, S)
    where
        E: Debug,
        F: Debug,
    {
        match self {
            Done(v, r) => (v, r),
            Fail(e, _) => panic!(
                "called ParseResult::unwrap on an Fail value; Error: {:?}.",
                e
            ),
            Fatal(e) => panic!(
                "called ParseResult::unwrap on an Fatal value; Error: {:?}.",
                e
            ),
        }
    }

    pub fn unwrap_err(self) -> (E, S)
    where
        O: Debug,
        F: Debug,
    {
        match self {
            Done(v, _) => panic!(
                "called ParseResult::unwrap_err on an Done value; Output: {:?}.",
                v
            ),
            Fatal(e) => panic!(
                "called ParseResult::unwrap_err on an Fatal value; Output: {:?}.",
                e
            ),
            Fail(e, r) => (e, r),
        }
    }

    pub fn as_ref(&self) -> ParseResult<&S, &O, &E, &F> {
        match self {
            Done(v, r) => Done(v, r),
            Fail(e, r) => Fail(e, r),
            Fatal(e) => Fatal(e),
        }
    }
    pub fn as_mut(&mut self) -> ParseResult<&mut S, &mut O, &mut E, &mut F> {
        match self {
            Done(v, r) => Done(v, r),
            Fail(e, r) => Fail(e, r),
            Fatal(e) => Fatal(e),
        }
    }

    pub fn result(self) -> Result<O, E, F> {
        match self {
            Done(v, _) => Ok(v),
            Fail(e, _) => Err(e),
            Fatal(e) => Fault(e),
        }
    }
}
