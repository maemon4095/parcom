use std::marker::PhantomData;

use parcom_core::{ParseResult::*, Parser, ParserResult};
pub struct Discard<S, P: Parser<S>> {
    parser: P,
    marker: PhantomData<S>,
}

impl<S, P: Parser<S>> Parser<S> for Discard<S, P> {
    type Output = ();
    type Error = P::Error;
    type Fault = P::Fault;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
        match self.parser.parse(input) {
            Done(_, r) => Done((), r),
            Fail(e, r) => Fail(e, r),
            Fatal(e) => Fatal(e),
        }
    }
}

impl<S, P: Parser<S>> Discard<S, P> {
    pub(super) fn new(parser: P) -> Self {
        Self {
            parser,
            marker: PhantomData,
        }
    }
}

pub struct DiscardErr<S, P: Parser<S>> {
    parser: P,
    marker: PhantomData<S>,
}

impl<S, P: Parser<S>> Parser<S> for DiscardErr<S, P> {
    type Output = P::Output;
    type Error = ();
    type Fault = P::Fault;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
        match self.parser.parse(input) {
            Done(e, r) => Done(e, r),
            Fail(_, r) => Fail((), r),
            Fatal(e) => Fatal(e),
        }
    }
}

impl<S, P: Parser<S>> DiscardErr<S, P> {
    pub(super) fn new(parser: P) -> Self {
        Self {
            parser,
            marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod test {
    use mockalloc::Mockalloc;
    use std::alloc::System;

    #[global_allocator]
    static ALLOCATOR: Mockalloc<System> = Mockalloc(System);

    use crate::parser::ParserExtension;
    use parcom_core::Parser;

    #[test]
    #[allow(unused_variables)]
    fn no_alloc() {
        let info = mockalloc::record_allocs(|| {
            let result = crate::primitive::str::atom_char(' ')
                .discard()
                .repeat(1..)
                .parse("        ");
        });

        println!("{:?}", info);

        assert_eq!(info.num_allocs(), 0)
    }
}
