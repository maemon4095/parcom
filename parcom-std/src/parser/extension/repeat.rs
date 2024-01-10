use crate::internal::just_on_boundary;
use parcom_core::{ParseResult::*, Parser, ParserResult, RewindStream};
use std::{marker::PhantomData, ops::RangeBounds};

pub struct Repeat<T: RewindStream, P: Parser<T>, R: RangeBounds<usize>> {
    pub(super) range: R,
    pub(super) parser: P,
    pub(super) marker: PhantomData<T>,
}

impl<S: RewindStream, P: Parser<S>, R: RangeBounds<usize>> Parser<S> for Repeat<S, P, R> {
    type Output = (Vec<P::Output>, Option<P::Error>);
    type Error = P::Error;
    type Fault = P::Fault;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
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
                    Fatal(e) => return Fatal(e),
                }
            };

            vec.push(e);
            rest = r;
        };

        if self.range.contains(&vec.len()) {
            Done((vec, last_error), rest)
        } else {
            Fail(last_error.unwrap(), rest.into())
        }
    }
}
