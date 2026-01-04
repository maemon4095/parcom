use super::control::{Control, Response};
use super::Node;
use crate::BufferStrategy;
use parcom_core::SequenceSegment;
use parcom_sequence_core::SequenceLoader;
use parcom_sequence_core::{LoadInfo, SequenceSource};
use pin_project::pin_project;
use std::future::Future;
use std::sync::{Arc, OnceLock};

pub struct GenericSequenceLoader<T, S: SequenceSource<Item = T>, B: BufferStrategy> {
    source: S,
    buf: Vec<T>,
    strategy: Arc<B>,
    tail_node: Arc<OnceLock<Node<T>>>,
}

impl<T, S: SequenceSource<Item = T>, B: BufferStrategy> GenericSequenceLoader<T, S, B> {
    pub(super) fn new(tail_node: Arc<OnceLock<Node<T>>>, source: S, strategy: Arc<B>) -> Self {
        Self {
            source,
            buf: Vec::new(),
            strategy,
            tail_node,
        }
    }
}

#[pin_project]
pub struct Load<'a, T, S, B>
where
    S: 'a + SequenceSource<Item = T>,
    B: BufferStrategy,
{
    #[pin]
    fut: S::Next<'a, Control<'a, T, S::Error, Arc<B>>>,
    tail_node: &'a mut Arc<OnceLock<Node<T>>>,
}

fn commit_to<T>(tail_node: &mut Arc<OnceLock<Node<T>>>, buf: Vec<T>) {
    let next = Arc::new(OnceLock::new());
    let r = tail_node.set(Node {
        buf,
        next: Arc::clone(&next),
    });
    assert!(r.is_ok());
    *tail_node = next;
}

impl<'a, T, S, B> Future for Load<'a, T, S, B>
where
    S: 'a + SequenceSource<Item = T>,
    B: BufferStrategy,
{
    type Output = Result<LoadInfo, S::Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        let res = std::task::ready!(this.fut.poll(cx));

        let r = match res {
            Response::Appended { len, cap } => Ok(LoadInfo::new(0, len, cap)),
            Response::Advance { buf, len, cap } => {
                let commited = buf.len();
                commit_to(this.tail_node, buf);
                Ok(LoadInfo::new(commited, len, cap))
            }
            Response::Finish(buf) => {
                let commited = buf.len();
                commit_to(this.tail_node, buf);
                Ok(LoadInfo::done(commited))
            }
            Response::Cancel(e) => Err(e),
        };
        std::task::Poll::Ready(r)
    }
}

impl<T, S, B> SequenceLoader for GenericSequenceLoader<T, S, B>
where
    S: SequenceSource<Item = T>,
    B: BufferStrategy,
{
    type Length = <[T] as SequenceSegment>::Length;
    type Segment = [T];
    type Error = S::Error;
    type Load<'a>
        = Load<'a, T, S, B>
    where
        Self: 'a;

    fn force_commit(&mut self) {
        let buf = std::mem::replace(&mut self.buf, Vec::new());
        commit_to(&mut self.tail_node, buf);
    }

    fn load(&mut self) -> Self::Load<'_> {
        let control = Control::new(&mut self.buf, &self.strategy);
        let fut = self.source.next(control, 0);
        let tail_node = &mut self.tail_node;
        Load { fut, tail_node }
    }
}
