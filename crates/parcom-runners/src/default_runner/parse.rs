use super::DefaultSequence;
use parcom_core::ParserOnce;
use parcom_runner_core::{RunnerError, SequenceLoaderRuntime};
use parcom_sequence_core::{SequenceBuffer, SequenceBuilder, SequenceSource};
use std::{future::Future, marker::PhantomData, task::Poll};

pub struct Parse<P, S, B> {
    _phantom: PhantomData<(P, S, B)>,
}

impl<B, P, S> Future for Parse<P, S, B>
where
    <<B as SequenceBuilder<S>>::Buffer as SequenceBuffer>::Length: Default + PartialEq,
    B: SequenceBuilder<S>,
    P: ParserOnce<DefaultSequence<S, B>>,
    S: SequenceSource,
{
    type Output = Result<P::Output, RunnerError<P::Error, S::Error>>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        todo!()
    }
}
