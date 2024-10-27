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

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let mut buf = std::array::from_fn(|_| MaybeUninit::uninit());
        let mut rest = input;
        let mut idx = 0;

        let result = loop {
            if idx >= buf.len() {
                return Done(buf.map(|e| unsafe { e.assume_init() }), rest);
            }

            let (v, r) = match self.parser.parse(rest).await {
                Done(v, r) => (v, r),
                Fail(e, r) => break Fail(e, r),
                Fatal(e, r) => break Fatal(e, r),
            };

            buf[idx] = MaybeUninit::new(v);
            rest = r;
            idx += 1;
        };

        for e in &mut buf[..idx] {
            unsafe { e.assume_init_drop() }
        }

        result
    }
}
