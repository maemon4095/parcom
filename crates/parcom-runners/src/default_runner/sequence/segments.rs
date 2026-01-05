use super::DefaultSequence;
use parcom_core::SegmentStream;
use parcom_internals::future::{
    notify::{self, Notify, Wait},
    option_future::OptionFuture,
};
use parcom_runner_core::{SequenceLoaderRuntime, SequenceLoaderRuntimeSession};
use parcom_sequence_core::{SequenceBuffer, SequenceBuilder, SequenceSource};
use pin_project::pin_project;
use std::{
    future::Future,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::Poll,
};

pub struct DefaultSegments<'a, S, B>
where
    S: SequenceSource,
    B: SequenceBuilder<S>,
    B::Buffer: 'a,
{
    append_signal: &'a Notify,
    done_flag: &'a AtomicBool,
    iter: <B::Buffer as SequenceBuffer>::Iter<'a>,
}

impl<'a, S, B> DefaultSegments<'a, S, B>
where
    S: SequenceSource,
    B: SequenceBuilder<S>,
{
    pub(super) fn new(sequence: &'a DefaultSequence<S, B>) -> Self {
        let iter = sequence.inner.buffer.segments();

        Self {
            iter,
            append_signal: &sequence.inner.append_signal,
            done_flag: &sequence.inner.done_flag,
        }
    }
}

impl<'a, S, B> SegmentStream for DefaultSegments<'a, S, B>
where
    S: SequenceSource,
    B: SequenceBuilder<S>,
    B::Segment: 'a,
{
    type Length = <B::Buffer as SequenceBuffer>::Length;
    type Segment = <B::Buffer as SequenceBuffer>::Segment;

    type Next<'b>
        = DefaultSegmentsNext<'a, 'b, S, B>
    where
        Self: 'b;

    fn next<'b>(&'b mut self, _: Self::Length) -> Self::Next<'b> {
        DefaultSegmentsNext {
            fut: OptionFuture::none(),
            host: self,
        }
    }
}

#[pin_project]
pub struct DefaultSegmentsNext<'a, 'b, S, B>
where
    S: SequenceSource,
    B: SequenceBuilder<S>,
    B::Segment: 'a,
{
    #[pin]
    fut: OptionFuture<Wait<'a>>,
    host: &'b mut DefaultSegments<'a, S, B>,
}

impl<'a, 'b, S, B> Future for DefaultSegmentsNext<'a, 'b, S, B>
where
    S: SequenceSource,
    B: SequenceBuilder<S>,
{
    type Output = Option<&'b <B::Buffer as SequenceBuffer>::Segment>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        std::task::ready!(this.fut.as_mut().poll(cx));

        if let Some(seg) = this.host.iter.next() {
            return Poll::Ready(Some(seg));
        }

        if this.host.done_flag.load(Ordering::SeqCst) {
            return Poll::Ready(None);
        }

        let fut = this.host.append_signal.wait();

        // `this.host.iter.next()`が`None`を返したあと、`this.host.append_signal.wait()`を呼ぶ前に次のセグメントが追加された場合のため、セグメントをチェックする。
        if let Some(seg) = this.host.iter.next() {
            return Poll::Ready(Some(seg));
        }

        this.fut.set(OptionFuture::some(fut));
        cx.waker().wake_by_ref();

        Poll::Pending
    }
}
