mod buffer;
mod control;
mod loader;

use crate::BufferStrategy;

use parcom_sequence_core::{SequenceBuilder, SequenceSource};
use std::sync::{Arc, OnceLock};

pub use buffer::{GenericSequenceBuffer, Iter};
pub use loader::{GenericSequenceLoader, Load};

struct Node<T> {
    buf: Vec<T>,
    next: Arc<OnceLock<Node<T>>>,
}

pub struct GenericSequenceBuilder<B: BufferStrategy> {
    strategy: Arc<B>,
}

impl<B: BufferStrategy, S: SequenceSource> SequenceBuilder<S> for GenericSequenceBuilder<B> {
    type Buffer = GenericSequenceBuffer<S::Item>;
    type Loader = GenericSequenceLoader<S::Item, S, B>;

    fn build(&self, source: S) -> (Self::Buffer, Self::Loader) {
        let node = Arc::new(OnceLock::new());
        let buffer = GenericSequenceBuffer::new(Arc::clone(&node));
        let loader = GenericSequenceLoader::new(node, source, Arc::clone(&self.strategy));
        (buffer, loader)
    }
}
