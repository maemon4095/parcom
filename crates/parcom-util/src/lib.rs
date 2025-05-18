mod either;
mod either_both;

pub mod error;
pub mod extension;

use parcom_core::{Error, ParseError, ParseResult, Stream, UnknownLocation};

pub use either::Either;
pub use either_both::EitherBoth;
pub use extension::{ParseResultExt, ResultExt};

pub fn done<S: Stream, O, E: ParseError>(output: O, rest: S) -> ParseResult<S, O, E> {
    Ok((output, rest))
}

pub fn fail<S: Stream, O, E: ParseError>(
    err: E,
    rest: impl Into<UnknownLocation<S>>,
) -> ParseResult<S, O, E> {
    Err(Error::Fail(err, rest.into()))
}
