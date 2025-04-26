use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum ParseResult<S: Stream, O, E: ParseError> {
    Done(O, S),
    Fail(E, UnknownLocation<S>),
    StreamErr(S::Error, UnknownLocation<S>),
}
use ParseResult::*;

use crate::{ParseError, Stream, UnknownLocation};

impl<S: Stream, O, E: ParseError> ParseResult<S, O, E> {
    pub fn map<T>(self, f: impl FnOnce(O) -> T) -> ParseResult<S, T, E> {
        match self {
            Done(v, r) => Done(f(v), r),
            Fail(e, r) => Fail(e, r),
            StreamErr(e, r) => StreamErr(e, r),
        }
    }

    pub fn map_err<T: ParseError>(self, f: impl FnOnce(E) -> T) -> ParseResult<S, O, T> {
        match self {
            Done(v, r) => Done(v, r),
            Fail(e, r) => Fail(f(e), r),
            StreamErr(e, r) => StreamErr(e, r),
        }
    }

    pub fn unwrap(self) -> (O, S)
    where
        E: Debug,
        S::Error: Debug,
    {
        match self {
            Done(v, r) => (v, r),
            Fail(e, _) => panic!(
                "called ParseResult::unwrap on an Fail value; Error: {:?}.",
                e
            ),
            StreamErr(e, _) => panic!(
                "called ParseResult::unwrap on an StreamError value; Error: {:?}.",
                e
            ),
        }
    }
}
