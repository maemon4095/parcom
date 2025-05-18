use parcom_core::{Error, ParseError, Stream};

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
