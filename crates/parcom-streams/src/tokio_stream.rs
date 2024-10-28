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
}

// pub struct Nodes<S: ?Sized> {}

// impl<S: ?Sized> parcom_core::ParcomStream for TokioStream<S> {
//     type Segment = S;
//     type Nodes = Nodes<S>;
//     type Advance;

//     fn segments(&self) -> Self::Nodes {
//         todo!()
//     }

//     fn advance(self, count: usize) -> Self::Advance {
//         todo!()
//     }
// }
