mod parse;
mod parse_iterative;
mod sequence;

use parcom_core::{IterativeParserOnce, ParserOnce};
use parcom_runner_core::{ParseRunner, RunnerError, SequenceLoaderRuntime};
use parcom_sequence_core::{SequenceBuilder, SequenceSource};
use std::marker::PhantomData;

pub use parse::Parse;
pub use parse_iterative::ParseIterative;
pub use sequence::{DefaultSegments, DefaultSegmentsNext, DefaultSequence};

pub struct DefaultRunner<S, B>
where
    S: SequenceSource,
    B: SequenceBuilder<S>,
{
    builder: B,
    _phantom: PhantomData<fn(S) -> ()>,
}

impl<S, B> ParseRunner<S> for DefaultRunner<S, B>
where
    B::Length: Default + PartialEq,
    S: SequenceSource,
    B: SequenceBuilder<S>,
{
    type Error<E> = RunnerError<E, S::Error>;
    type Sequence = DefaultSequence<S, B>;
    type Parse<P: ParserOnce<Self::Sequence>> = Parse<P, S, B>;
    type ParseIterative<P: IterativeParserOnce<Self::Sequence>> = ParseIterative<P, S, B>;

    fn parse<P>(&self, parser: P, source: S) -> Self::Parse<P>
    where
        S: SequenceSource,
        P: ParserOnce<Self::Sequence>,
    {
        todo!()
    }

    fn parse_iterative<P>(&self, parser: P, source: S) -> Self::ParseIterative<P>
    where
        P: IterativeParserOnce<Self::Sequence>,
    {
        todo!()
    }
}
