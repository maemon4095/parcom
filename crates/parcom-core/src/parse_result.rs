use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum ParseResult<S, O, E, F = crate::Never> {
    Done(O, S),
    Fail(E, UnknownLocation<S>),
    Fatal(F),
}

use ParseResult::*;

use crate::UnknownLocation;

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
    pub fn as_ref(&self) -> ParseResult<&S, &O, &E, &F> {
        match self {
            Done(v, r) => Done(v, r),
            Fail(e, r) => Fail(e, r.as_ref()),
            Fatal(e) => Fatal(e),
        }
    }
}
