use std::{marker::PhantomData, ops::RangeBounds};

use crate::{
    ParseResult::{self, *},
    Parser, RewindStream,
};

use crate::standard::just_on_boundary;

pub struct Repeat<T: RewindStream, P: Parser<T>, R: RangeBounds<usize>> {
    pub(super) range: R,
    pub(super) parser: P,
    pub(super) marker: PhantomData<T>,
}

impl<S: RewindStream, P: Parser<S>, R: RangeBounds<usize>> Parser<S> for Repeat<S, P, R> {
    type Output = Vec<P::Output>;
    type Error = P::Error;

    fn parse(&self, input: S) -> ParseResult<S, Self::Output, Self::Error> {
        let mut vec = Vec::new();
        let upper_bound = self.range.end_bound();

        let mut rest = input;
        let (last_error, rest) = loop {
            if just_on_boundary(vec.len(), upper_bound) {
                break (None, rest);
            }

            let (e, r) = {
                let anchor = rest.anchor();
                match self.parser.parse(rest) {
                    Done(v, r) => (v, r),
                    Fail(e, r) => break (Some(e), r.rewind(anchor)),
                }
            };

            vec.push(e);
            rest = r;
        };

        if self.range.contains(&vec.len()) {
            Done(vec, rest)
        } else {
            Fail(last_error.unwrap(), rest)
        }
    }
}
