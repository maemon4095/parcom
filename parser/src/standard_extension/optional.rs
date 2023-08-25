use std::marker::PhantomData;

use crate::Parser;

pub struct Optional<T, P: Parser<T>> {
    pub(super) parser: P,
    pub(super) marker: PhantomData<T>,
}

impl<T, P: Parser<T>> Parser<T> for Optional<T, P> {
    type Output = Option<P::Output>;
    type Error = ();

    fn parse<S: crate::ParseStream<Item = T>>(
        &self,
        input: S,
    ) -> Result<(Self::Output, S), (Self::Error, S)> {
        let anchor = input.anchor();
        match self.parser.parse(input) {
            Ok((v, r)) => Ok((Some(v), r)),
            Err((_, r)) => Ok((None, r.rewind(anchor))),
        }
    }
}
