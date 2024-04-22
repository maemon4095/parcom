use parcom_core::{ParseResult::*, Parser, ParserResult, RewindStream};
use std::{marker::PhantomData, mem::MaybeUninit};

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
                Fail(e, r) => return Fail(e, r),
                Fatal(e, r) => return Fatal(e, r),
            };

            *elem = MaybeUninit::new(v);
            rest = r;
        }

        Done(unsafe { buf.map(|e| e.assume_init()) }, rest)
    }
}
