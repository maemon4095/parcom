use std::fmt::Debug;

#[derive(Debug, Clone, Copy)]
pub enum ParseResult<S, O, E> {
    Done(O, S),
    Fail(E, S),
}

impl<S, O, E> ParseResult<S, O, E> {
    pub fn map<T>(self, f: impl FnOnce(O) -> T) -> ParseResult<S, T, E> {
        use ParseResult::*;
        match self {
            Done(v, r) => Done(f(v), r),
            Fail(e, r) => Fail(e, r),
        }
    }

    pub fn map_err<T>(self, f: impl FnOnce(E) -> T) -> ParseResult<S, O, T> {
        use ParseResult::*;
        match self {
            Done(v, r) => Done(v, r),
            Fail(e, r) => Fail(f(e), r),
        }
    }

    pub fn and_then<T>(self, f: impl FnOnce(O) -> Result<T, E>) -> ParseResult<S, T, E> {
        match self {
            ParseResult::Done(v, r) => match f(v) {
                Ok(v) => ParseResult::Done(v, r),
                Err(e) => ParseResult::Fail(e, r),
            },
            ParseResult::Fail(e, r) => ParseResult::Fail(e, r),
        }
    }

    pub fn or_else<T>(self, f: impl FnOnce(E) -> Result<O, T>) -> ParseResult<S, O, T> {
        match self {
            ParseResult::Done(v, r) => ParseResult::Done(v, r),
            ParseResult::Fail(e, r) => match f(e) {
                Ok(v) => ParseResult::Done(v, r),
                Err(e) => ParseResult::Fail(e, r),
            },
        }
    }

    pub fn ok(self) -> Option<O> {
        match self {
            ParseResult::Done(v, _) => Some(v),
            ParseResult::Fail(_, _) => None,
        }
    }

    pub fn err(self) -> Option<E> {
        match self {
            ParseResult::Done(_, _) => None,
            ParseResult::Fail(e, _) => Some(e),
        }
    }

    pub fn unwrap(self) -> (O, S)
    where
        E: Debug,
    {
        match self {
            ParseResult::Done(v, r) => (v, r),
            ParseResult::Fail(e, _) => panic!(
                "called ParseResult::unwrap on an Err value; Error: {:?}.",
                e
            ),
        }
    }

    pub fn unwrap_err(self) -> (E, S)
    where
        O: Debug,
    {
        match self {
            ParseResult::Done(v, _) => panic!(
                "called ParseResult::unwrap_err on an Ok value; Output: {:?}.",
                v
            ),
            ParseResult::Fail(e, r) => (e, r),
        }
    }

    pub fn unwrap_or(self, default: O) -> (O, S) {
        match self {
            ParseResult::Done(v, r) => (v, r),
            ParseResult::Fail(_, r) => (default, r),
        }
    }

    pub fn unwrap_or_else(self, default: impl FnOnce(E) -> O) -> (O, S) {
        match self {
            ParseResult::Done(v, r) => (v, r),
            ParseResult::Fail(e, r) => (default(e), r),
        }
    }

    pub fn as_ref(&self) -> ParseResult<&S, &O, &E> {
        use ParseResult::*;
        match self {
            Done(v, r) => Done(v, r),
            Fail(e, r) => Fail(e, r),
        }
    }
    pub fn as_mut(&mut self) -> ParseResult<&mut S, &mut O, &mut E> {
        use ParseResult::*;
        match self {
            Done(v, r) => Done(v, r),
            Fail(e, r) => Fail(e, r),
        }
    }

    pub fn result(self) -> Result<O, E> {
        match self {
            ParseResult::Done(v, _) => Ok(v),
            ParseResult::Fail(e, _) => Err(e),
        }
    }

    pub fn cloned(self) -> ParseResult<S, O, E>
    where
        O: Clone,
    {
        match self {
            ParseResult::Done(v, r) => ParseResult::Done(v.clone(), r),
            e @ _ => e,
        }
    }

    pub fn into_result(self) -> Result<(O, S), (E, S)> {
        match self {
            ParseResult::Done(v, r) => Ok((v, r)),
            ParseResult::Fail(e, r) => Err((e, r)),
        }
    }
}
