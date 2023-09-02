use std::marker::PhantomData;

use crate::{Parser, RewindStream};

pub struct Optional<T: RewindStream, P: Parser<T>> {
    pub(super) parser: P,
    pub(super) marker: PhantomData<T>,
}

impl<S: RewindStream, P: Parser<S>> Parser<S> for Optional<S, P> {
    type Output = Option<P::Output>;
    type Error = ();

    fn parse(&self, input: S) -> Result<(Self::Output, S), (Self::Error, S)> {
        let anchor = input.anchor();
        match self.parser.parse(input) {
            Ok((v, r)) => Ok((Some(v), r)),
            Err((_, r)) => Ok((None, r.rewind(anchor))),
        }
    }
}
