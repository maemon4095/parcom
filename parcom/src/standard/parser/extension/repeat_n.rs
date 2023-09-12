use std::{marker::PhantomData, mem::MaybeUninit};

use crate::{ParseResult::*, Parser, ParserResult, RewindStream};

pub struct RepeatN<T: RewindStream, P: Parser<T>, const N: usize> {
    pub(super) parser: P,
    pub(super) marker: PhantomData<T>,
}
impl<S: RewindStream, P: Parser<S>, const N: usize> Parser<S> for RepeatN<S, P, N> {
    type Output = [P::Output; N];
    type Error = P::Error;
    type Fault = P::Fault;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
        let mut buf = std::array::from_fn(|_| MaybeUninit::uninit());

        let mut rest = input;
        for elem in buf.iter_mut() {
            let (v, r) = match self.parser.parse(rest) {
                Done(v, r) => (v, r),
                Fail(v, r) => return Fail(v, r),
                Fatal(e) => return Fatal(e),
            };

            *elem = MaybeUninit::new(v);
            rest = r;
        }

        Done(unsafe { buf.map(|e| e.assume_init()) }, rest)
    }
}
