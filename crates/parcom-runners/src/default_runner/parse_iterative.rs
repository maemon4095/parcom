use super::DefaultSequence;
use parcom_core::IterativeParserOnce;
use parcom_runner_core::{IterativeParseSession, RunnerError, SequenceLoaderRuntime};
use parcom_sequence_core::{SequenceBuilder, SequenceSource};
use std::{marker::PhantomData, task::Poll};

pub struct ParseIterative<P, S, B, R>
where
    P: IterativeParserOnce<DefaultSequence<S, B, R>>,
    S: SequenceSource,
    B: SequenceBuilder<S>,
    R: SequenceLoaderRuntime<B::Loader>,
    B::Length: Default + PartialEq,
{
    _phantom: PhantomData<(P, S, B, R)>,
}

impl<P, S, B, R> IterativeParseSession for ParseIterative<P, S, B, R>
where
    P: IterativeParserOnce<DefaultSequence<S, B, R>>,
    S: SequenceSource,
    B: SequenceBuilder<S>,
    R: SequenceLoaderRuntime<B::Loader>,
    B::Length: Default + PartialEq,
{
    type Output = P::Output;
    type Error = RunnerError<P::Error, S::Error>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<Option<Self::Output>, Self::Error>> {
        todo!()
    }
}
