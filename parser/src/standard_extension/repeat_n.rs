use std::{marker::PhantomData, mem::MaybeUninit};

use crate::Parser;

pub struct RepeatN<T, P: Parser<T>, const N: usize> {
    pub(super) parser: P,
    pub(super) marker: PhantomData<T>,
}
impl<T, P: Parser<T>, const N: usize> Parser<T> for RepeatN<T, P, N> {
    type Output = [P::Output; N];
    type Error = P::Error;

    fn parse<S: crate::ParseStream<Item = T>>(
        &self,
        input: S,
    ) -> Result<(Self::Output, S), (Self::Error, S)> {
        let mut buf = std::array::from_fn(|_| MaybeUninit::uninit());

        let mut rest = input;
        for elem in buf.iter_mut() {
            let (v, r) = match self.parser.parse(rest) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            *elem = MaybeUninit::new(v);
            rest = r;
        }

        Ok((unsafe { buf.map(|e| e.assume_init()) }, rest))
    }
}
