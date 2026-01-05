mod advance;
mod segments;

use parcom_core::Sequence;
use parcom_internals::future::notify::Notify;
use parcom_runner_core::SequenceLoaderRuntime;
use parcom_sequence_core::{SequenceBuffer, SequenceBuilder, SequenceSource};
use std::sync::{atomic::AtomicBool, Arc};

pub use advance::DefaultSequenceAdvance;
pub use segments::{DefaultSegments, DefaultSegmentsNext};

pub struct DefaultSequence<S, B>
where
    S: SequenceSource,
    B: SequenceBuilder<S>,
{
    inner: Box<DefaultSequenceInner<S, B>>,
}

impl<S, B> DefaultSequence<S, B>
where
    S: SequenceSource,
    B: SequenceBuilder<S>,
{
    pub fn new(buffer: B::Buffer, loader: B::Loader) -> Self {
        Self {
            inner: Box::new(DefaultSequenceInner {
                buffer,
                loader,
                append_signal: Arc::new(Notify::new()),
                done_flag: Arc::new(AtomicBool::new(false)),
            }),
        }
    }
}

struct DefaultSequenceInner<S, B>
where
    S: SequenceSource,
    B: SequenceBuilder<S>,
{
    buffer: B::Buffer,
    loader: B::Loader,
    append_signal: Arc<Notify>,
    done_flag: Arc<AtomicBool>,
}

impl<S, B> Sequence for DefaultSequence<S, B>
where
    S: SequenceSource,
    B: SequenceBuilder<S>,
    B::Length: Default + PartialEq,
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
