use std::{
    future::{Future, IntoFuture},
    marker::PhantomData,
    sync::Arc,
    task::Poll,
};

use crate::generic_stream::{IntermediateNode, Node, NodePtr};
use parcom_streams_core::{BufferWriter, LoadInfo, StreamControl, StreamLoader, StreamSource};
use pin_project::pin_project;

pub struct GenericStreamLoader<T, S>
where
    S: StreamSource<Item = T>,
{
    min_capacity: usize,
    source: S,
    buf: Vec<T>,
    tail: NodePtr<T>,
}

impl<T, S> GenericStreamLoader<T, S>
where
    S: StreamSource<Item = T>,
{
    pub(super) fn new(source: S, tail: NodePtr<T>) -> GenericStreamLoader<T, S> {
        Self {
            min_capacity: 0,
            source,
            buf: Vec::new(),
            tail,
        }
    }
}

impl<T, S> StreamLoader for GenericStreamLoader<T, S>
where
    S: StreamSource<Item = T>,
{
    type Error = S::Error;
    type Load<'a>
        = Load<'a, T, S>
    where
        Self: 'a;

    fn set_min_buffer_size(&mut self, size: usize) {
        self.min_capacity = size;
    }

    fn force_commit(&mut self) {
        if self.buf.is_empty() {
            return;
        }

        let tail_next = match &*self.tail {
            Node::Head(next) => next,
            Node::Intermediate(node) => &node.next,
            Node::Sentinel => return,
        };

        let buf = std::mem::replace(&mut self.buf, Vec::new());
        let node = IntermediateNode::new(buf);
        let node_ptr = Arc::new(Node::Intermediate(node));

        let result = tail_next.set(Arc::clone(&node_ptr));
        assert!(result.is_ok());

        self.tail = node_ptr;
    }

    fn load(&mut self) -> Self::Load<'_> {
        let control = Control {
            min_capacity: self.min_capacity,
            pre_allocated: &mut self.buf,
            phantom: PhantomData,
        };

        Load {
            min_capacity: self.min_capacity,
            tail: &mut self.tail,
            buf: &mut self.buf,
            fut: self.source.next(control, 0).into_future(),
        }
    }
}

#[pin_project]
pub struct Load<'a, T, S>
where
    S: 'a + StreamSource<Item = T>,
{
    min_capacity: usize,
    tail: &'a mut NodePtr<T>,
    buf: *mut Vec<T>,
    #[pin]
    fut: <S as StreamSource>::Next<'a, Control<'a, T, <S as StreamSource>::Error>>,
}

impl<'a, T, S> Future for Load<'a, T, S>
where
    S: 'a + StreamSource<Item = T>,
{
    type Output = Result<Option<LoadInfo>, S::Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut this = self.project();

        let tail_next = match &***this.tail {
            Node::Head(next) => next,
            Node::Intermediate(node) => &node.next,
            Node::Sentinel => return Poll::Ready(Ok(None)),
        };
        let result = std::task::ready!(this.fut.as_mut().poll(cx));

        match result {
            WriterResult::PreAllocated => {
                let buf = unsafe { &**this.buf };
                let capacity = buf.capacity();
                let uncommited = buf.len();
                let info = LoadInfo::new(0, uncommited, capacity);
                Poll::Ready(Ok(Some(info)))
            }
            WriterResult::Allocated(next_buf) => {
                let uncommited = next_buf.len();
                let capacity = next_buf.capacity();
                let buf = unsafe { std::ptr::replace(*this.buf, next_buf) };
                let commited = buf.len();

                let next = IntermediateNode::new(buf);
                let next_ptr = Arc::new(Node::Intermediate(next));
                let result = tail_next.set(next_ptr.clone());
                assert!(result.is_ok());
                **this.tail = next_ptr;

                let info = LoadInfo::new(commited, uncommited, capacity);

                Poll::Ready(Ok(Some(info)))
            }
            WriterResult::Finish => {
                let buf = unsafe { std::ptr::replace(*this.buf, Vec::new()) };
                let committed = buf.len();
                let sentinel = Arc::new(Node::Sentinel);
                let next = IntermediateNode::new_with(buf, Arc::clone(&sentinel));
                let next_ptr = Arc::new(Node::Intermediate(next));

                let result = tail_next.set(Arc::clone(&next_ptr));
                assert!(result.is_ok());

                **this.tail = sentinel;

                let info = LoadInfo::new(committed, 0, 0);

                Poll::Ready(Ok(Some(info)))
            }
            WriterResult::Error(e) => Poll::Ready(Err(e)),
        }
    }
}

struct Control<'a, T: 'a, E> {
    min_capacity: usize,
    pre_allocated: *mut Vec<T>,
    phantom: PhantomData<(&'a mut (), E)>,
}

impl<'a, T, E> StreamControl for Control<'a, T, E> {
    type Item = T;
    type Result = WriterResult<T, E>;
    type Error = E;
    type Writer = Writer<'a, T, E>;

    fn request_writer(self, min_capacity: usize) -> Self::Writer {
        let Control { pre_allocated, .. } = self;
        let pre_allocated = unsafe { &mut *pre_allocated };
        let pre_allocated_cap = pre_allocated.capacity() - pre_allocated.len();

        if min_capacity <= pre_allocated_cap {
            let ptr = unsafe { pre_allocated.as_mut_ptr().add(pre_allocated.len()) };
            Writer {
                pre_allocated: Some(pre_allocated),
                ptr,
                len: 0,
                cap: pre_allocated_cap,
                phantom: PhantomData,
            }
        } else {
            let mut buf = Vec::with_capacity(min_capacity.max(self.min_capacity));
            let ptr = buf.as_mut_ptr();
            let cap = buf.capacity();
            Vec::leak(buf);
            Writer {
                pre_allocated: None,
                ptr,
                len: 0,
                cap,
                phantom: PhantomData,
            }
        }
    }

    fn cancel(self, err: Self::Error) -> Self::Result {
        WriterResult::Error(err)
    }

    fn finish(self) -> Self::Result {
        WriterResult::Finish
    }
}

struct Writer<'a, T, E> {
    pre_allocated: Option<&'a mut Vec<T>>,
    ptr: *mut T,
    len: usize,
    cap: usize,
    phantom: PhantomData<E>,
}

enum WriterResult<T, E> {
    PreAllocated,
    Allocated(Vec<T>),
    Finish,
    Error(E),
}

impl<'a, T, E> BufferWriter for Writer<'a, T, E> {
    type Item = T;
    type Result = WriterResult<T, E>;
    type Error = E;

    fn capacity(&self) -> usize {
        self.cap
    }

    fn len(&self) -> usize {
        self.len
    }

    fn as_ptr(&self) -> *const Self::Item {
        self.ptr
    }

    fn as_mut_ptr(&mut self) -> *mut Self::Item {
        self.ptr
    }

    unsafe fn set_len(&mut self, new_len: usize) {
        assert!(new_len <= self.cap);
        self.len = new_len;
    }

    fn advance(self) -> Self::Result {
        match self.pre_allocated {
            Some(v) => unsafe {
                v.set_len(v.len() + self.len);
                WriterResult::PreAllocated
            },
            None => unsafe {
                let buf = Vec::from_raw_parts(self.ptr, self.len, self.cap);
                WriterResult::Allocated(buf)
            },
        }
    }

    fn cancel(self, err: Self::Error) -> Self::Result {
        WriterResult::Error(err)
    }
}
