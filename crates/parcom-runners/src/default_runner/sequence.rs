mod advance;
mod segments;

use parcom_core::Sequence;
use parcom_internals::future::notify::Notify;
use parcom_runner_core::SequenceLoaderRuntime;
use parcom_sequence_core::{SequenceBuffer, SequenceBuilder, SequenceSource};
use std::sync::{atomic::AtomicBool, Arc};

pub use advance::DefaultSequenceAdvance;
pub use segments::{DefaultSegments, DefaultSegmentsNext};

pub struct DefaultSequence<S, B, R>
where
    S: SequenceSource,
    B: SequenceBuilder<S>,
    R: SequenceLoaderRuntime<B::Loader>,
{
    inner: Box<DefaultSequenceInner<S, B, R>>,
}

impl<S, B, R> DefaultSequence<S, B, R>
where
    S: SequenceSource,
    B: SequenceBuilder<S>,
    R: SequenceLoaderRuntime<B::Loader>,
{
    pub fn new(buffer: B::Buffer, session: R::Session) -> Self {
        Self {
            inner: Box::new(DefaultSequenceInner { buffer, session }),
        }
    }
}

struct DefaultSequenceInner<S, B, R>
where
    S: SequenceSource,
    B: SequenceBuilder<S>,
    R: SequenceLoaderRuntime<B::Loader>,
{
    buffer: B::Buffer,
    session: R::Session,
}

impl<S, B, R> Sequence for DefaultSequence<S, B, R>
where
    S: SequenceSource,
    B: SequenceBuilder<S>,
    R: SequenceLoaderRuntime<B::Loader>,
    B::Length: Default + PartialEq,
{
    type Length = <B::Buffer as SequenceBuffer>::Length;
    type Segment = <B::Buffer as SequenceBuffer>::Segment;

    type Segments<'a>
        = DefaultSegments<'a, S, B, R>
    where
        Self: 'a;

    type Advance = DefaultSequenceAdvance<S, B, R>;

    fn segments<'a>(&'a mut self) -> Self::Segments<'a> {
        DefaultSegments::new(self)
    }

    fn advance(self, delta: Self::Length) -> Self::Advance {
        DefaultSequenceAdvance::new(self, delta)
    }
}
