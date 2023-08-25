use std::{marker::PhantomData, ops::RangeBounds};

use crate::{ParseStream, Parser};

use super::just_on_boundary;

pub struct Repeat<T, P: Parser<T>, R: RangeBounds<usize>> {
    pub(super) range: R,
    pub(super) parser: P,
    pub(super) marker: PhantomData<T>,
}

impl<T, P: Parser<T>, R: RangeBounds<usize>> Parser<T> for Repeat<T, P, R> {
    type Output = Vec<P::Output>;
    type Error = P::Error;

    fn parse<S: ParseStream<Item = T>>(
        &self,
        input: S,
    ) -> Result<(Self::Output, S), (Self::Error, S)> {
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
                    Ok(t) => t,
                    Err((e, r)) => break (Some(e), r.rewind(anchor)),
                }
            };

            vec.push(e);
            rest = r;
        };

        if self.range.contains(&vec.len()) {
            Ok((vec, rest))
        } else {
            Err((last_error.unwrap(), rest))
        }
    }
}
