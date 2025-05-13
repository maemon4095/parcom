mod never;
mod parser;
mod stream;
mod unknown;

pub mod parse_error;
pub mod primitive;

pub use measured;
pub use never::{Never, ShouldNever, ShouldNeverExtension};
pub use parse_error::{Error, ParseError};
pub use parser::*;
pub use stream::*;
pub use unknown::UnknownLocation;

pub type ParserResult<S, P> =
    ParseResult<S, <P as ParserOnce<S>>::Output, <P as ParserOnce<S>>::Error>;

pub type ParseResult<S, O, E> = Result<(O, S), Error<S, E>>;
