mod never;
mod parse_result;
mod parser;
mod stream;
mod unknown;

pub mod measured;
pub mod parse_error;
pub mod primitive;

pub use never::{Never, ShouldNever, ShouldNeverExtension};
pub use parse_error::ParseError;
pub use parse_result::ParseResult;
pub use parser::*;
pub use stream::*;
pub use unknown::UnknownLocation;

pub type ParserResult<S, P> = ParseResult<S, <P as Parser<S>>::Output, <P as Parser<S>>::Error>;
