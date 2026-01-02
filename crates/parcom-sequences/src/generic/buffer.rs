use parcom_core::SequenceSegment;
use parcom_sequence_core::SequenceBuffer;
use std::sync::{Arc, OnceLock};

use super::Node;

pub struct GenericSequenceBuffer<T> {
    head_offset: usize,
    head_node: Arc<OnceLock<Node<T>>>,
}

impl<T> GenericSequenceBuffer<T> {
    pub(super) fn new(head_node: Arc<OnceLock<Node<T>>>) -> Self {
        Self {
            head_offset: 0,
            head_node,
        }
    }
}

impl<T> SequenceBuffer for GenericSequenceBuffer<T> {
    type Length = <[T] as SequenceSegment>::Length;
    type Segment = [T];
    type Iter<'a>
        = Iter<'a, T>
    where
        Self: 'a;

    fn advance(
        &mut self,
        length: <Self::Segment as SequenceSegment>::Length,
    ) -> <Self::Segment as SequenceSegment>::Length {
        let mut remain = length;
        let mut offset = self.head_offset;
        let mut node = &self.head_node;

        while let Some(n) = node.get() {
            let len = n.buf.len() - offset;
            if remain < len {
                offset += remain;
                remain = 0;
                break;
            }

            remain -= len;
            offset = 0;
            node = &n.next;
        }

        self.head_offset = offset;
        self.head_node = Arc::clone(&node);

        remain
    }

    fn segments(&self) -> Self::Iter<'_> {
        Iter {
            offset: self.head_offset,
            node: &self.head_node,
        }
    }
}
pub struct Iter<'a, T> {
    offset: usize,
    node: &'a Arc<OnceLock<Node<T>>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.node.get()?;
        let seg = &node.buf[self.offset..];
        let next = &node.next;
        self.offset = 0;
        self.node = next;
        Some(seg)
    }
}
