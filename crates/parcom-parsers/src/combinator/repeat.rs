use parcom_core::{
    Error, IterativeParser, IterativeParserOnce, IterativeParserState, ParseError, ParseResult,
    Parser, ParserOnce, RewindSequence, Sequence,
};
use parcom_util::{done, fail, ResultExt};
use std::marker::PhantomData;

use super::Ref;

#[derive(Debug)]
pub struct Repeat<S: RewindSequence, P: Parser<S>> {
    parser: P,
    marker: PhantomData<S>,
}

impl<S: RewindSequence, P: Parser<S>> Repeat<S, P> {
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            marker: PhantomData,
        }
    }
}

impl<S: RewindSequence, P: Parser<S>> ParserOnce<S> for Repeat<S, P> {
    type Output = (Vec<P::Output>, P::Error);
    type Error = P::Error;

    async fn parse_once(self, input: S) -> parcom_core::ParserResult<S, Self> {
        self.parse(input).await
    }
}

impl<S: RewindSequence, P: Parser<S>> Parser<S> for Repeat<S, P> {
    async fn parse(&self, input: S) -> ParseResult<S, Self::Output, Self::Error> {
        let mut buf = Vec::new();
        let mut rest = input;

        let last_err = loop {
            let anchor = rest.anchor();
            match self.parser.parse(rest).await {
                Ok((v, r)) => {
                    buf.push(v);
                    rest = r;
                }
                Err(Error::Fail(e, r)) => {
                    if e.should_terminate() {
                        return fail(e, r);
                    }

                    rest = r.rewind(anchor).await.stream_err()?;
                    break e;
                }
                Err(e) => return Err(e),
            }
        };

        done((buf, last_err), rest)
    }
}

impl<S: RewindSequence, P: Parser<S>> IterativeParserOnce<S> for Repeat<S, P> {
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

impl<S: RewindSequence, P: Parser<S>> IterativeParser<S> for Repeat<S, P> {
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
pub struct IterationState<S: Sequence, P: Parser<S>> {
    parser: P,
    marker: PhantomData<S>,
}

impl<S: RewindSequence, P: Parser<S>> IterativeParserState<S> for IterationState<S, P> {
    type Output = P::Output;
    type Error = P::Error;

    async fn parse_next(&mut self, input: S) -> ParseResult<S, Option<Self::Output>, Self::Error> {
        let anchor = input.anchor();
        match self.parser.parse(input).await {
            Ok((v, r)) => done(Some(v), r),
            Err(Error::Fail(e, r)) => {
                if e.should_terminate() {
                    fail(e, r)
                } else {
                    done(None, r.rewind(anchor).await.stream_err()?)
                }
            }
            Err(e) => return Err(e),
        }
    }
}
