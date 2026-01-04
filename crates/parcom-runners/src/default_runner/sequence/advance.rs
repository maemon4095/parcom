use super::{DefaultSequence, DefaultSequenceInner};
use parcom_internals::future::{notify::Wait, option_future::OptionFuture};
use parcom_runner_core::{SequenceLoaderRuntime, SequenceLoaderRuntimeSession};
use parcom_sequence_core::{SequenceBuffer, SequenceBuilder, SequenceSource};
use pin_project::pin_project;
use std::sync::atomic::Ordering;
use std::{future::Future, sync::Arc, task::Poll};
#[pin_project]
pub struct DefaultSequenceAdvance<
    S: SequenceSource,
    B: SequenceBuilder<S>,
    R: SequenceLoaderRuntime<B::Loader>,
> {
    // DO NOT CHANGE THE FIELD ORDER
    // `fut` must be dropped before `sequence` be dropped.
    #[pin]
    fut: OptionFuture<
        <R::Session as SequenceLoaderRuntimeSession<B::Loader>>::WaitForAppend<'static>,
    >,

    sequence: Option<Box<DefaultSequenceInner<S, B, R>>>,
    remain: <B::Buffer as SequenceBuffer>::Length,
}

impl<S: SequenceSource, B: SequenceBuilder<S>, R: SequenceLoaderRuntime<B::Loader>>
    DefaultSequenceAdvance<S, B, R>
{
    pub(super) fn new(
        sequence: DefaultSequence<S, B, R>,
        delta: <B::Buffer as SequenceBuffer>::Length,
    ) -> Self {
        Self {
            fut: OptionFuture::none(),
            sequence: Some(sequence.inner),
            remain: delta,
        }
    }
}

impl<S: SequenceSource, B: SequenceBuilder<S>, R: SequenceLoaderRuntime<B::Loader>> Future
    for DefaultSequenceAdvance<S, B, R>
where
    B::Length: Default + PartialEq,
{
    type Output = DefaultSequence<S, B, R>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        let sequence = this.sequence.as_mut().unwrap();
        std::task::ready!(this.fut.as_mut().poll(cx));

        // SAFETY: `DefaultSequenceInner`のインスタンスは`self`と同じ期間生存し、フィールド順により`fut`のほうが先にdropされるため、dangling参照が発生することはない。
        let fut: <R::Session as SequenceLoaderRuntimeSession<B::Loader>>::WaitForAppend<'static> =
            unsafe { sequence.session.wait_for_append() };
        this.fut.set(OptionFuture::some(fut));

        let remain = std::mem::replace(this.remain, Default::default());
        let remain = sequence.buffer.advance(remain);

        if &remain == this.remain || sequence.done_flag.load(Ordering::SeqCst) {
            // `sequence`をtakeする前に`fut`がdropされることを保証する。
            this.fut.set(OptionFuture::none());
            // `this.remain == Default::default()`であるため、`remain == Default::default()`つまり`remain`が0になっている。
            // よってadvanceが完了している。
            let sequence = this.sequence.take().unwrap();
            return Poll::Ready(DefaultSequence { inner: sequence });
        }

        cx.waker().wake_by_ref();

        Poll::Pending
    }
}
