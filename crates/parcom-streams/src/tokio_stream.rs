mod once_init;

use once_init::OnceInit;
use std::{mem::MaybeUninit, sync::atomic::AtomicBool};
use tokio::sync::mpsc::{channel, Receiver, Sender};

/// ```text
///   [T]            [T]            [T]
///    ↑              ↑              ↑
///   [node][ptr] -> [node][ptr] -> [node][null]
///    ↑
/// [anchor]
/// ```
pub struct TokioStream<S: ?Sized> {
    receiver: Receiver<Box<S>>,
    head: Node<S>,
    tail: Node<S>,
}

struct Node<S: ?Sized> {
    segment: Box<S>,
    next: OnceInit<Option<Self>>,
}

pub struct Segments<S: ?Sized> {
    node: Node<S>,
}

impl<S: ?Sized> parcom_core::ParcomStream for TokioStream<S> {
    type Segment = S;
    type SegmentStream = Segments<S>;
    type Advance;

    fn segments(&self) -> Self::SegmentStream {
        todo!()
    }

    fn advance(self, count: usize) -> Self::Advance {
        todo!()
    }
}
