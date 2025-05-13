mod either;
mod either_both;

pub mod error;

use parcom_core::{Error, ParseError, ParseResult, Stream, UnknownLocation};

pub use either::Either;
pub use either_both::EitherBoth;

pub fn done<S: Stream, O, E: ParseError>(output: O, rest: S) -> ParseResult<S, O, E> {
    Ok((output, rest))
}

pub fn fail<S: Stream, O, E: ParseError>(
    err: E,
    rest: impl Into<UnknownLocation<S>>,
) -> ParseResult<S, O, E> {
    Err(parcom_core::Error::Fail(err, rest.into()))
}

pub trait ParseResultExt {
    type Stream: Stream;
    type Output;
    type Error: ParseError;

    fn map_fail<U: ParseError>(
        self,
        f: impl FnOnce(Self::Error) -> U,
    ) -> ParseResult<Self::Stream, Self::Output, U>;

    fn conv_fail<U: ParseError + From<Self::Error>>(
        self,
    ) -> ParseResult<Self::Stream, Self::Output, U>;
}
impl<S: Stream, O, E: ParseError> ParseResultExt for Result<(O, S), Error<S, E>> {
    type Stream = S;
    type Output = O;
    type Error = E;

    fn map_fail<U: ParseError>(
        self,
        f: impl FnOnce(Self::Error) -> U,
    ) -> ParseResult<Self::Stream, Self::Output, U> {
        self.map_err(|e| e.map_fail(f))
    }

    fn conv_fail<U: ParseError + From<Self::Error>>(
        self,
    ) -> ParseResult<Self::Stream, Self::Output, U> {
        self.map_err(|e| e.conv_fail())
    }
}

pub trait ResultExt: Sized {
    type Ok;
    type Err;

    fn stream_err<S, E>(self) -> Result<Self::Ok, Error<S, E>>
    where
        S: Stream<Error = Self::Err>,
        E: ParseError;
}

impl<O, E> ResultExt for Result<O, E> {
    type Ok = O;
    type Err = E;

    fn stream_err<S, U>(self) -> Result<O, Error<S, U>>
    where
        S: Stream<Error = E>,
        U: ParseError,
    {
        self.map_err(Error::Stream)
    }
}
