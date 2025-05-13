use crate::{Stream, UnknownLocation};

#[derive(Debug, Clone)]
pub enum Error<S: Stream, E: ParseError> {
    Fail(E, UnknownLocation<S>),
    Stream(S::Error),
}

impl<S: Stream, E: ParseError> Error<S, E> {
    pub fn conv_fail<U: ParseError + From<E>>(self) -> Error<S, U> {
        match self {
            Error::Fail(e, r) => Error::Fail(e.into(), r),
            Error::Stream(e) => Error::Stream(e),
        }
    }

    pub fn map_fail<U: ParseError>(self, f: impl FnOnce(E) -> U) -> Error<S, U> {
        match self {
            Error::Fail(e, r) => Error::Fail(f(e), r),
            Error::Stream(e) => Error::Stream(e),
        }
    }
}

pub trait ParseError {
    fn should_terminate(&self) -> bool;
}

impl<T: ParseError> ParseError for &T {
    fn should_terminate(&self) -> bool {
        T::should_terminate(self)
    }
}

impl<T: ParseError> ParseError for Option<T> {
    fn should_terminate(&self) -> bool {
        match self {
            Some(v) => v.should_terminate(),
            None => false,
        }
    }
}
