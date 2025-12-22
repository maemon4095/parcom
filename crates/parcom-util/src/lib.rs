mod either;
mod either_both;

pub mod error;

use parcom_core::{ParseError, ParseResult, Sequence, UnknownLocation};

pub use either::Either;
pub use either_both::EitherBoth;

pub fn done<S: Sequence, O, E: ParseError>(output: O, rest: S) -> ParseResult<S, O, E> {
    Ok((output, rest))
}

pub fn fail<S: Sequence, O, E: ParseError>(
    err: impl Into<E>,
    rest: impl Into<UnknownLocation<S>>,
) -> ParseResult<S, O, E> {
    Err((err.into(), rest.into()))
}
