use parcom_core::{
    IterativeParser, IterativeParserOnce, IterativeParserState, ParseError, ParseResult, Parser,
    ParserOnce, RewindStream,
};
use std::marker::PhantomData;

use super::Ref;

#[derive(Debug)]
pub struct Repeat<S: RewindStream, P: Parser<S>> {
    parser: P,
    marker: PhantomData<S>,
}

impl<S: RewindStream, P: Parser<S>> Repeat<S, P> {
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            marker: PhantomData,
        }
    }
}

impl<S: RewindStream, P: Parser<S>> ParserOnce<S> for Repeat<S, P> {
    type Output = (Vec<P::Output>, P::Error);
    type Error = P::Error;

    async fn parse_once(self, input: S) -> parcom_core::ParserResult<S, Self> {
        self.parse(input).await
    }
}

impl<S: RewindStream, P: Parser<S>> Parser<S> for Repeat<S, P> {
    async fn parse(&self, input: S) -> ParseResult<S, Self::Output, Self::Error> {
        let mut buf = Vec::new();
        let mut rest = input;

        let last_err = loop {
            let anchor = rest.anchor();
            match self.parser.parse(rest).await {
                ParseResult::Done(v, r) => {
                    buf.push(v);
                    rest = r;
                }
                ParseResult::Fail(e, r) => {
                    if e.should_terminate() {
                        return ParseResult::Fail(e, r);
                    }

                    rest = r.rewind(anchor).await;
                    break e;
                }
            }
        };

        ParseResult::Done((buf, last_err), rest)
    }
}

impl<S: RewindStream, P: Parser<S>> IterativeParserOnce<S> for Repeat<S, P> {
    type Output = P::Output;
    type Error = P::Error;
    type StateOnce = IterationState<S, P>;

    fn start_once(self) -> Self::StateOnce {
        IterationState {
            parser: self.parser,
            marker: PhantomData,
        }
    }
}

impl<S: RewindStream, P: Parser<S>> IterativeParser<S> for Repeat<S, P> {
    type State<'a>
        = IterationState<S, Ref<'a, S, P>>
    where
        Self: 'a;
    fn start(&self) -> Self::State<'_> {
        IterationState {
            parser: Ref::new(&self.parser),
            marker: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct IterationState<S, P: Parser<S>> {
    parser: P,
    marker: PhantomData<S>,
}

impl<S: RewindStream, P: Parser<S>> IterativeParserState<S> for IterationState<S, P> {
    type Output = P::Output;
    type Error = P::Error;

    async fn parse_next(&mut self, input: S) -> ParseResult<S, Option<Self::Output>, Self::Error> {
        let anchor = input.anchor();
        match self.parser.parse(input).await {
            ParseResult::Done(v, r) => ParseResult::Done(Some(v), r),
            ParseResult::Fail(e, r) => {
                if e.should_terminate() {
                    ParseResult::Fail(e, r)
                } else {
                    ParseResult::Done(None, r.rewind(anchor).await)
                }
            }
        }
    }
}
