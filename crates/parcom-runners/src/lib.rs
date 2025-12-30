use std::marker::PhantomData;

use parcom_core::{IterativeParserOnce, ParserOnce};
use parcom_runner_core::ParseRunner;
use parcom_sequence_core::{SequenceBuilder, SequenceSource};

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
    S: SequenceSource,
    B: SequenceBuilder<S>,
{
    type Sequence = Sequence<S>;
    type Parse<P>
        = Parse<P>
    where
        P: ParserOnce<Self::Sequence>,
        S: SequenceSource;

    type IterationSession<P>
        = IterationSession<P>
    where
        P: IterativeParserOnce<Self::Sequence>,
        S: SequenceSource;

    fn parse<P>(&self, parser: P, source: S) -> Self::Parse<P>
    where
        P: ParserOnce<Self::Sequence>,
    {
        todo!()
    }

    fn start_parse<P>(&self, parser: P, source: S) -> Self::IterationSession<P>
    where
        P: IterativeParserOnce<Self::Sequence>,
        S: SequenceSource,
    {
        todo!()
    }
}

pub struct Sequence<S> {
    _phansom: PhantomData<S>,
}

pub struct Parse<P> {
    _phantom: PhantomData<P>,
}

pub struct IterationSession<P> {
    _phantom: PhantomData<P>,
}
