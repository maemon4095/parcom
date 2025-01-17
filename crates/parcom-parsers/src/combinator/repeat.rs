use parcom_core::{
    IterativeParser, IterativeParserState, Never, ParseError, ParseResult, Parser, RewindStream,
};
use std::marker::PhantomData;

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

impl<S: RewindStream, P: Parser<S>> Parser<S> for Repeat<S, P> {
    type Output = (Vec<P::Output>, P::Error);
    type Error = P::Error;

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

impl<S: RewindStream, P: Parser<S>> IterativeParser<S> for Repeat<S, P> {
    type Output = P::Output;
    type Error = P::Error;
    type PrerequisiteError = Never;
    type State<'a>
        = IterationState<'a, S, P>
    where
        Self: 'a;

    fn start(&self) -> Self::State<'_> {
        IterationState { me: self }
    }
}

#[derive(Debug)]
pub struct IterationState<'a, S: RewindStream, P: Parser<S>> {
    me: &'a Repeat<S, P>,
}

impl<'a, S: RewindStream, P: Parser<S>> IterativeParserState<S> for IterationState<'a, S, P> {
    type Output = P::Output;
    type Error = P::Error;
    type PrerequisiteError = Never;

    fn prerequisite_error(&self) -> Option<Self::PrerequisiteError> {
        None
    }

    async fn parse_next(
        &mut self,
        input: S,
    ) -> ParseResult<S, Result<Self::Output, Self::Error>, Self::Error> {
        let anchor = input.anchor();
        match self.me.parser.parse(input).await {
            ParseResult::Done(v, r) => ParseResult::Done(Ok(v), r),
            ParseResult::Fail(e, r) => {
                if e.should_terminate() {
                    ParseResult::Fail(e, r)
                } else {
                    ParseResult::Done(Err(e), r.rewind(anchor).await)
                }
            }
        }
    }
}
