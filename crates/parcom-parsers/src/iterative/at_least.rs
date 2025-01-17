use parcom_base::Either;
use parcom_core::{
    IterativeParser, IterativeParserState, ParseError,
    ParseResult::{self, *},
    Parser,
};
use std::marker::PhantomData;

use super::Collect;

#[derive(Debug)]
pub struct AtLeast<S, P: IterativeParser<S>> {
    parser: P,
    count: usize,
    marker: PhantomData<S>,
}

impl<S, P: IterativeParser<S>> AtLeast<S, P> {
    pub fn new(parser: P, count: usize) -> Self {
        Self {
            parser,
            count,
            marker: PhantomData,
        }
    }
}

impl<S, P: IterativeParser<S>> Parser<S> for AtLeast<S, P> {
    type Output = (Vec<P::Output>, P::Error);
    type Error = Either<P::Error, Either<InsufficientCountError, P::PrerequisiteError>>;

    async fn parse(&self, input: S) -> ParseResult<S, Self::Output, Self::Error> {
        Collect::new(self).parse(input).await
    }
}

impl<S, P: IterativeParser<S>> IterativeParser<S> for AtLeast<S, P> {
    type Output = P::Output;
    type Error = P::Error;
    type PrerequisiteError = Either<InsufficientCountError, P::PrerequisiteError>;

    type State<'a>
        = IterationState<S, P::State<'a>>
    where
        Self: 'a;

    fn start(&self) -> Self::State<'_> {
        IterationState {
            state: self.parser.start(),
            requied: self.count,
            insufficient: self.count,
            marker: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct IterationState<S, P: IterativeParserState<S>> {
    state: P,
    requied: usize,
    insufficient: usize,
    marker: PhantomData<S>,
}

#[derive(Debug)]
pub struct InsufficientCountError {
    requied: usize,
    insufficent: usize,
}

impl InsufficientCountError {
    pub fn requied(&self) -> usize {
        self.requied
    }
    pub fn insufficent(&self) -> usize {
        self.insufficent
    }
}

impl ParseError for InsufficientCountError {
    fn should_terminate(&self) -> bool {
        false
    }
}

impl std::fmt::Display for InsufficientCountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Insufficient item count. At least {} items are required and {} are missing.",
            self.requied, self.insufficent
        )
    }
}

impl<S, P: IterativeParserState<S>> IterativeParserState<S> for IterationState<S, P> {
    type Output = P::Output;
    type Error = P::Error;
    type PrerequisiteError = Either<InsufficientCountError, P::PrerequisiteError>;

    fn prerequisite_error(&self) -> Option<Self::PrerequisiteError> {
        if let Some(e) = self.state.prerequisite_error() {
            return Some(Either::Last(e));
        }

        if self.insufficient > 0 {
            Some(Either::First(InsufficientCountError {
                requied: self.requied,
                insufficent: self.insufficient,
            }))
        } else {
            None
        }
    }

    async fn parse_next(
        &mut self,
        input: S,
    ) -> ParseResult<S, Result<Self::Output, Self::Error>, Self::Error> {
        match self.state.parse_next(input).await {
            Done(v, r) => {
                if self.insufficient > 0 {
                    self.insufficient -= 1;
                }
                Done(v, r)
            }
            Fail(e, r) => Fail(e, r),
        }
    }
}
