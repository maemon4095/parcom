use parcom_core::{Error, ParseError, ParseResult, Sequence};

pub trait ParseResultExt {
    type Stream: Sequence;
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

impl<S: Sequence, O, E: ParseError> ParseResultExt for Result<(O, S), Error<S, E>> {
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
