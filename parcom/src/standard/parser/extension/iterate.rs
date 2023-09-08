use std::marker::PhantomData;

use crate::{
    standard::parser::iterate::iterate,
    ParseResult::{self, *},
    Parser, RewindStream,
};

use super::ParserExtension;
pub(super) use internal::Iter;
pub struct Iterate<S: RewindStream, P: Parser<S>, O, E, F: Fn(&mut Iter<S, P>) -> Result<O, E>> {
    pub(super) parser: P,
    pub(super) op: F,
    pub(super) marker: PhantomData<(S, O, E)>,
}

impl<S: RewindStream, P: Parser<S>, O, E, F: Fn(&mut Iter<S, P>) -> Result<O, E>> Parser<S>
    for Iterate<S, P, O, E, F>
{
    type Output = O;
    type Error = E;

    fn parse(&self, input: S) -> ParseResult<S, Self::Output, Self::Error> {
        let mut iter = Iter(iterate(input, self.parser.as_ref()));
        let result = (self.op)(&mut iter);
        let (rest, _, _) = iter.0.deconstruct();

        match result {
            Ok(v) => Done(v, rest),
            Err(e) => Fail(e, rest),
        }
    }
}

mod internal {
    use crate::standard::parser::{iterate, AsRef};
    use crate::{Parser, RewindStream};

    pub struct Iter<'a, S: RewindStream, P: Parser<S>>(
        pub(super) iterate::Iter<S, AsRef<'a, S, P>>,
    );

    impl<'a, S: RewindStream, P: Parser<S>> Iter<'a, S, P> {
        pub fn last_err(&self) -> Option<&P::Error> {
            self.0.last_err()
        }

        pub fn next(&mut self) -> Result<P::Output, P::Error> {
            self.0.next()
        }
    }

    impl<'a, S: RewindStream, P: Parser<S>> Iterator for Iter<'a, S, P> {
        type Item = Result<P::Output, P::Error>;

        fn next(&mut self) -> Option<Self::Item> {
            Some(self.next())
        }
    }
}
