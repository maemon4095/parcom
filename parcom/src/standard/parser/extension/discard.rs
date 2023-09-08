use std::marker::PhantomData;

use crate::{
    ParseResult::{self, *},
    Parser,
};
pub struct Discard<S, P: Parser<S>> {
    pub(super) parser: P,
    pub(super) marker: PhantomData<S>,
}

impl<S, P: Parser<S>> Parser<S> for Discard<S, P> {
    type Output = ();
    type Error = P::Error;

    fn parse(&self, input: S) -> ParseResult<S, Self::Output, Self::Error> {
        match self.parser.parse(input) {
            Done(_, r) => Done((), r),
            Fail(e, r) => Fail(e, r),
        }
    }
}

#[cfg(test)]
mod test {
    use mockalloc::Mockalloc;
    use std::alloc::System;

    #[global_allocator]
    static ALLOCATOR: Mockalloc<System> = Mockalloc(System);

    use crate::standard::parser::ParserExtension;
    use crate::Parser;

    #[test]
    #[allow(unused_variables)]
    fn no_alloc() {
        let info = mockalloc::record_allocs(|| {
            let result = crate::foreign::parser::str::atom_char(' ')
                .discard()
                .repeat(1..)
                .parse("        ");
        });

        println!("{:?}", info);

        assert_eq!(info.num_allocs(), 0)
    }
}
