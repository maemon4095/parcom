use std::marker::PhantomData;

use parcom_core::{
    IterativeParser, IterativeParserOnce, IterativeParserState, ParseResult, Parser, ParserOnce,
};

pub struct Fold<S, P, A, F>
where
    P: IterativeParserOnce<S>,
    F: Fn(A, P::Output) -> A,
{
    parser: P,
    init: A,
    f: F,
    marker: PhantomData<S>,
}

impl<S, P, A, F> Fold<S, P, A, F>
where
    P: IterativeParserOnce<S>,
    F: Fn(A, P::Output) -> A,
{
    pub fn new(parser: P, init: A, f: F) -> Self {
        Self {
            parser,
            init,
            f,
            marker: PhantomData,
        }
    }
}

impl<S, P, A, F> ParserOnce<S> for Fold<S, P, A, F>
where
    P: IterativeParserOnce<S>,
    F: Fn(A, P::Output) -> A,
{
    type Output = A;
    type Error = P::Error;

    fn parse_once(
        self,
        input: S,
    ) -> impl std::future::Future<Output = parcom_core::ParserResult<S, Self>> {
        parse(self.parser.start_once(), self.init, self.f, input)
    }
}

impl<S, P, A, F> Parser<S> for Fold<S, P, A, F>
where
    P: IterativeParser<S>,
    A: Clone,
    F: Fn(A, P::Output) -> A,
{
    fn parse(
        &self,
        input: S,
    ) -> impl std::future::Future<Output = parcom_core::ParserResult<S, Self>> {
        parse(self.parser.start(), self.init.clone(), &self.f, input)
    }
}

async fn parse<S, P, A, F>(mut state: P, init: A, f: F, input: S) -> ParseResult<S, A, P::Error>
where
    P: IterativeParserState<S>,
    F: Fn(A, P::Output) -> A,
{
    let mut acc = init;
    let mut rest = input;
    loop {
        match state.parse_next(rest).await {
            ParseResult::Done(Some(v), r) => {
                acc = f(acc, v);
                rest = r;
            }
            ParseResult::Done(None, r) => return ParseResult::Done(acc, r),
            ParseResult::Fail(e, r) => return ParseResult::Fail(e, r),
        }
    }
}
