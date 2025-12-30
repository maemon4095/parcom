mod never;
mod parse_error;
mod parser;
mod sequence;
mod unknown;

pub mod primitive;

pub use measured;
pub use never::{Never, ShouldNever, ShouldNeverExtension};
pub use parse_error::ParseError;
pub use parser::*;
pub use sequence::*;
pub use unknown::UnknownLocation;

pub type ParserResult<S, P> =
    ParseResult<S, <P as ParserOnce<S>>::Output, <P as ParserOnce<S>>::Error>;

pub type ParseResult<S, O, E> = Result<(O, S), (E, UnknownLocation<S>)>;
