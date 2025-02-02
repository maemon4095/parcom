use parcom_core::Parser;
use std::marker::PhantomData;

pub struct Boxed<S, P: Parser<S>> {
    parser: P,
    marker: PhantomData<S>,
}

impl<S, P: Parser<S>> Boxed<S, P> {
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            marker: PhantomData,
        }
    }
}

impl<S, P: Parser<S>> Parser<S> for Boxed<S, P> {
    type Output = P::Output;
    type Error = P::Error;

    fn parse(
        &self,
        input: S,
    ) -> impl std::future::Future<Output = parcom_core::ParseResult<S, Self::Output, Self::Error>>
    {
        Box::pin(self.parser.parse(input))
    }
}
