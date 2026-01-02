mod advance;
mod segments;

use parcom_core::Sequence;
use parcom_internals::future::notify::Notify;
use parcom_sequence_core::{SequenceBuffer, SequenceBuilder, SequenceSource};
use std::sync::{atomic::AtomicBool, Arc};

pub use advance::DefaultSequenceAdvance;
pub use segments::{DefaultSegments, DefaultSegmentsNext};

pub struct DefaultSequence<S: SequenceSource, B: SequenceBuilder<S>> {
    inner: Box<DefaultSequenceInner<S, B>>,
}

impl<S: SequenceSource, B: SequenceBuilder<S>> DefaultSequence<S, B> {
    pub fn new(buffer: B::Buffer, done_flag: Arc<AtomicBool>, append_signal: Arc<Notify>) -> Self {
        Self {
            inner: Box::new(DefaultSequenceInner {
                buffer,
                done_flag,
                append_signal,
            }),
        }
    }
}

struct DefaultSequenceInner<S: SequenceSource, B: SequenceBuilder<S>> {
    buffer: B::Buffer,
    done_flag: Arc<AtomicBool>,
    append_signal: Arc<Notify>,
}

impl<S: SequenceSource, B: SequenceBuilder<S>> Sequence for DefaultSequence<S, B>
where
    <<B as SequenceBuilder<S>>::Buffer as SequenceBuffer>::Length: Default + PartialEq,
{
    type Length = <B::Buffer as SequenceBuffer>::Length;
    type Segment = <B::Buffer as SequenceBuffer>::Segment;

    type Segments<'a>
        = DefaultSegments<'a, S, B>
    where
        Self: 'a;

    type Advance = DefaultSequenceAdvance<S, B>;

    fn segments<'a>(&'a mut self) -> Self::Segments<'a> {
        DefaultSegments::new(self)
    }

    fn advance(self, delta: Self::Length) -> Self::Advance {
        DefaultSequenceAdvance::new(self, delta)
    }
}
