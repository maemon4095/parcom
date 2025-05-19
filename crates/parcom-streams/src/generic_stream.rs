use std::{
    cell::{OnceCell, UnsafeCell},
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::Poll,
};

use crate::stream_control::Response;
use parcom_core::{SegmentIterator, Stream, StreamSegment};
use parcom_streams_core::{BufferRequest, StreamControl, StreamSource};

#[derive(Debug)]
pub struct GenericStream<T: 'static + Default, S: StreamSource<Segment = [T]>> {
    parameter: GenericStreamParameter,
    source: S,
    head_offset: usize,
    tail_len: usize,
    pair: Option<(NodeSlot<T>, NodeSlot<T>)>,
}

struct Control<'a, T, E> {
    reserved: &'a mut [T],
    parameter: &'a GenericStreamParameter,
    _phantom: PhantomData<fn(E) -> E>,
}

impl<'a, T: Default, E> StreamControl for Control<'a, T, E> {
    type Segment = [T];
    type Response = Response<Result<usize, (Vec<T>, usize)>, E>;
    type Error = E;
    type Request = Request<'a, T, E>;

    fn request_buffer(self, min_size: usize) -> Self::Request {
        if self.reserved.len() >= min_size {
            Request::Reserved {
                reserved: self.reserved,
                _phantom: PhantomData,
            }
        } else {
            let capacity = self.parameter.calc_new_segment_capacity(min_size);

            Request::Allocated {
                buf: std::iter::repeat_with(Default::default)
                    .take(capacity)
                    .collect(),
                _phantom: PhantomData,
            }
        }
    }

    fn cancel(self, err: Self::Error) -> Self::Response {
        Response::Cancel(Ok(0), err)
    }

    fn finish(self) -> Self::Response {
        Response::Finish(Ok(0))
    }
}

enum Request<'a, T, E> {
    Reserved {
        reserved: &'a mut [T],
        _phantom: PhantomData<fn(E) -> E>,
    },
    Allocated {
        buf: Vec<T>,
        _phantom: PhantomData<fn(E) -> E>,
    },
}

impl<'a, T: Default, E> BufferRequest for Request<'a, T, E> {
    type Control = Control<'a, T, E>;

    fn buffer(&mut self) -> &mut <Self::Control as StreamControl>::Segment {
        match self {
            Request::Reserved { reserved, .. } => reserved,
            Request::Allocated { buf, .. } => buf,
        }
    }

    fn advance(self, written: usize) -> <Self::Control as StreamControl>::Response {
        match self {
            Request::Reserved { reserved, .. } => {
                assert!(written <= reserved.len(),);
                Response::Advance(Ok(written))
            }
            Request::Allocated { buf, _phantom } => {
                assert!(written <= buf.len(),);

                Response::Advance(Err((buf, written)))
            }
        }
    }

    fn cancel(
        self,
        err: <Self::Control as StreamControl>::Error,
    ) -> <Self::Control as StreamControl>::Response {
        Response::Cancel(Ok(0), err)
    }
}

impl<T: 'static + Default, S: StreamSource<Segment = [T]>> GenericStream<T, S> {
    pub const fn new(source: S, parameter: GenericStreamParameter) -> Self {
        Self {
            parameter,
            source,
            head_offset: 0,
            tail_len: 0,
            pair: None,
        }
    }

    async fn load_next(&mut self, size_hint: usize) -> Result<Option<&[T]>, S::Error> {
        let Some((_, tail_slot)) = &mut self.pair else {
            let control = Control {
                reserved: &mut [],
                parameter: &self.parameter,
                _phantom: PhantomData,
            };
            let res = self.source.next(control, size_hint).await;
            match res {
                Response::Advance(result) => {
                    let (segment, len) = match result {
                        Ok(_) => {
                            let cap = self.parameter.min_capacity;
                            let buf = std::iter::repeat_with(Default::default).take(cap).collect();
                            (buf, 0)
                        }
                        Err(p) => p,
                    };

                    let node = NodeSlot::Some(Arc::new(Node {
                        segment: segment.into(),
                        next: OnceCell::new(),
                    }));

                    self.tail_len = len;

                    let (_, tail) = self.pair.get_or_insert((node.clone(), node));

                    let NodeSlot::Some(tail) = tail else {
                        unreachable!()
                    };
                    return Ok(Some(&tail.segment()[..len]));
                }
                Response::Cancel(_, e) => return Err(e),
                Response::Finish(_) => {
                    self.pair = Some((NodeSlot::Terminal, NodeSlot::Terminal));
                    return Ok(None);
                }
            }
        };

        let NodeSlot::Some(tail) = tail_slot else {
            return Ok(None);
        };

        let control = Control {
            reserved: &mut tail.segment_mut()[self.tail_len..],
            parameter: &self.parameter,
            _phantom: PhantomData,
        };
        let res = self.source.next(control, size_hint).await;

        match res {
            Response::Advance(Err((segment, len))) => {
                let next = NodeSlot::Some(Arc::new(Node {
                    segment: UnsafeCell::new(segment),
                    next: OnceCell::new(),
                }));
                unsafe {
                    (*tail.segment.get()).drain(self.tail_len..);
                }
                self.tail_len = len;
                let result = tail.next.set(next.clone());
                debug_assert!(result.is_ok());

                let (_, tail) = self.pair.as_mut().unwrap();
                *tail = next;

                let NodeSlot::Some(tail) = tail else {
                    unreachable!()
                };
                Ok(Some(tail.segment()))
            }
            Response::Advance(Ok(written)) => {
                let (_, tail) = self.pair.as_ref().unwrap();
                let NodeSlot::Some(tail) = tail else {
                    unreachable!()
                };
                let last_len = self.tail_len;
                self.tail_len += written;
                Ok(Some(&tail.segment()[last_len..self.tail_len]))
            }
            Response::Cancel(_, e) => Err(e),
            Response::Finish(_) => {
                let result = tail.next.set(NodeSlot::Terminal);
                debug_assert!(result.is_ok());
                let (_, tail) = self.pair.as_mut().unwrap();
                *tail = NodeSlot::Terminal;
                Ok(None)
            }
        }
    }
}

#[derive(Debug)]
pub struct GenericStreamParameter {
    min_capacity: usize,       // セグメントの最小容量
    unused_ratio_limit: usize, // セグメント内で利用されていない部分の割合の上限。利用されていない部分の割合がこの値を上回った場合、セグメントは縮小される。
}

impl Default for GenericStreamParameter {
    fn default() -> Self {
        Self {
            min_capacity: 16,
            unused_ratio_limit: 0,
        }
    }
}

impl GenericStreamParameter {
    fn calc_new_segment_capacity(&self, min_size: usize) -> usize {
        usize::max(min_size, self.min_capacity)
    }
}

#[derive(Debug)]
enum NodeSlot<T: 'static + Default> {
    Some(Arc<Node<T>>),
    Terminal,
}
impl<T: 'static + Default> Clone for NodeSlot<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Some(r) => Self::Some(Arc::clone(&r)),
            Self::Terminal => Self::Terminal,
        }
    }
}

impl<T: 'static + Default> NodeSlot<T> {
    fn is_terminal(&self) -> bool {
        match self {
            NodeSlot::Some(_) => false,
            NodeSlot::Terminal => true,
        }
    }
}

#[derive(Debug)]
struct Node<T: 'static + Default> {
    segment: UnsafeCell<Vec<T>>,
    next: OnceCell<NodeSlot<T>>,
}

impl<T: 'static + Default> Node<T> {
    fn segment(&self) -> &[T] {
        unsafe { (*self.segment.get()).as_slice() }
    }

    fn segment_mut(&self) -> &mut [T] {
        unsafe { (*self.segment.get()).as_mut_slice() }
    }
}

pub struct Next<'a, T: 'static + Default, S: 'a + StreamSource<Segment = [T]>> {
    fut: Pin<Box<dyn 'a + Future<Output = Result<Option<&'a S::Segment>, S::Error>>>>,
}

impl<'a, T: 'static + Default, S: 'a + StreamSource<Segment = [T]>> Future for Next<'a, T, S> {
    type Output = Result<Option<&'a S::Segment>, S::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        unsafe { self.map_unchecked_mut(|s| &mut s.fut).poll(cx) }
    }
}

pub struct Iter<'a, T: 'static + Default, S: 'a + StreamSource<Segment = [T]>> {
    stream: &'a mut GenericStream<T, S>,
    state: IterState<T>,
}

enum IterState<T: 'static + Default> {
    Initial,
    Iterating(NodeSlot<T>),
    Loading,
}

impl<'a, T: 'static + Default, S: StreamSource<Segment = [T]>> SegmentIterator for Iter<'a, T, S> {
    type Segment = [T];
    type Error = S::Error;
    type Next<'b>
        = Next<'b, T, S>
    where
        Self: 'b;

    fn next(&mut self, size_hint: <Self::Segment as StreamSegment>::Length) -> Self::Next<'_> {
        match &self.state {
            IterState::Iterating(slot) => match slot {
                NodeSlot::Some(node) => {
                    let next = node.next.get();
                    match next {
                        Some(next) => {
                            self.state = IterState::Iterating(next.clone());
                            let IterState::Iterating(node) = &self.state else {
                                unreachable!()
                            };

                            match node {
                                NodeSlot::Some(node) => {
                                    let is_tail = node.next.get().is_none_or(|x| x.is_terminal());
                                    let segment = node.segment();

                                    let len = if is_tail {
                                        self.stream.tail_len
                                    } else {
                                        segment.len()
                                    };

                                    Next {
                                        fut: Box::pin(async move { Ok(Some(&segment[..len])) }),
                                    }
                                }
                                NodeSlot::Terminal => Next {
                                    fut: Box::pin(async { Ok(None) }),
                                },
                            }
                        }
                        None => {
                            self.state = IterState::Loading;

                            Next {
                                fut: Box::pin(
                                    async move { self.stream.load_next(size_hint).await },
                                ),
                            }
                        }
                    }
                }
                NodeSlot::Terminal => Next {
                    fut: Box::pin(async { Ok(None) }),
                },
            },
            IterState::Initial => match &self.stream.pair {
                Some((head, _)) => {
                    self.state = IterState::Iterating(head.clone());
                    let IterState::Iterating(node) = &self.state else {
                        unreachable!()
                    };

                    let fut: Pin<Box<dyn Future<Output = _>>> = match node {
                        NodeSlot::Some(node) => {
                            let offset = self.stream.head_offset;
                            let segment = node.segment();
                            let is_tail = node.next.get().is_none_or(|n| n.is_terminal());

                            let len = if is_tail {
                                self.stream.tail_len
                            } else {
                                segment.len()
                            };

                            Box::pin(async move { Ok(Some(&segment[offset..len])) })
                        }
                        NodeSlot::Terminal => Box::pin(async { Ok(None) }),
                    };

                    Next { fut }
                }
                None => {
                    self.state = IterState::Loading;
                    Next {
                        fut: Box::pin(async move { self.stream.load_next(size_hint).await }),
                    }
                }
            },
            IterState::Loading => Next {
                fut: Box::pin(async move { self.stream.load_next(size_hint).await }),
            },
        }
    }
}

pub struct Advance<T: 'static + Default, S: 'static + StreamSource<Segment = [T]>> {
    fut: Pin<Box<dyn Future<Output = Result<GenericStream<T, S>, S::Error>>>>,
}

impl<T: 'static + Default, S: 'static + StreamSource<Segment = [T]>> Future for Advance<T, S> {
    type Output = Result<GenericStream<T, S>, S::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        unsafe { self.map_unchecked_mut(|f| &mut f.fut).poll(cx) }
    }
}

impl<T: 'static + Default, S: 'static + StreamSource<Segment = [T]>> Stream
    for GenericStream<T, S>
{
    type Segment = [T];
    type Error = S::Error;
    type SegmentIter<'a>
        = Iter<'a, T, S>
    where
        Self: 'a;
    type Advance = Advance<T, S>;

    fn segments(&mut self) -> Self::SegmentIter<'_> {
        Iter {
            stream: self,
            state: IterState::Initial,
        }
    }

    fn advance(self, delta: <Self::Segment as StreamSegment>::Length) -> Self::Advance {
        todo!()
    }
}

async fn advance_load<T: 'static + Default, S: 'static + StreamSource<Segment = [T]>>(
    mut source: S,
    mut remain: usize,
) -> Result<GenericStream<T, S>, S::Error> {
    todo!()
}

#[cfg(test)]
mod test {
    use crate::stream_source::iterator_source::IteratorSource;

    use super::*;

    #[test]
    fn test_load_all() {
        let array: &[&[usize]] = &[&[0], &[1, 2, 3], &[], &[4, 5, 6]];
        let source = IteratorSource::new(array);
        let mut stream = GenericStream::new(source, Default::default());

        pollster::block_on(async {
            let expected: Vec<_> = array.iter().flat_map(|s| s.iter().copied()).collect();
            let mut expected = expected.as_slice();
            let mut segments = stream.segments();

            while let Some(segment) = segments.next(0).await.unwrap() {
                match expected.strip_prefix(segment) {
                    Some(rest) => {
                        expected = rest;
                    }
                    None => unreachable!(),
                }
            }
            assert!(expected.is_empty())
        });
    }

    #[test]
    fn test_loaded_all() {
        let array: &[&[usize]] = &[&[0], &[1, 2, 3], &[], &[4, 5, 6]];
        let source = IteratorSource::new(array);
        let mut stream = GenericStream::new(source, Default::default());

        pollster::block_on(async {
            let expected: Vec<_> = array.iter().flat_map(|s| s.iter().copied()).collect();
            let mut expected = expected.as_slice();
            let mut segments = stream.segments();

            while let Some(segment) = segments.next(0).await.unwrap() {
                match expected.strip_prefix(segment) {
                    Some(rest) => {
                        expected = rest;
                    }
                    None => unreachable!(),
                }
            }
            assert!(expected.is_empty())
        });

        pollster::block_on(async {
            let expected: Vec<_> = array.iter().flat_map(|s| s.iter().copied()).collect();
            let mut expected = expected.as_slice();
            let mut segments = stream.segments();
            while let Some(segment) = segments.next(0).await.unwrap() {
                match expected.strip_prefix(segment) {
                    Some(rest) => {
                        expected = rest;
                    }
                    None => unreachable!(),
                }
            }
            assert!(expected.is_empty())
        });
    }

    #[test]
    fn test_loaded_partial() {
        let array: &[&[usize]] = &[&[0], &[1, 2, 3], &[], &[4, 5, 6]];
        let source = IteratorSource::new(array);
        let mut stream = GenericStream::new(source, Default::default());

        pollster::block_on(async {
            let take = 2;
            let expected: Vec<_> = array
                .iter()
                .take(take)
                .flat_map(|s| s.iter().copied())
                .collect();
            let mut expected = expected.as_slice();
            let mut segments = stream.segments();

            let mut left = take;
            while let Some(segment) = segments.next(0).await.unwrap() {
                match expected.strip_prefix(segment) {
                    Some(rest) => {
                        expected = rest;
                    }
                    None => unreachable!(),
                }

                left -= 1;
                if left == 0 {
                    break;
                }
            }
            assert!(expected.is_empty())
        });

        pollster::block_on(async {
            let expected: Vec<_> = array.iter().flat_map(|s| s.iter().copied()).collect();
            let mut expected = expected.as_slice();
            let mut segments = stream.segments();
            while let Some(segment) = segments.next(0).await.unwrap() {
                match expected.strip_prefix(segment) {
                    Some(rest) => {
                        expected = rest;
                    }
                    None => unreachable!(),
                }
            }
            assert!(expected.is_empty())
        });
    }

    #[test]
    fn test_advance_fully_loaded_advance_all() {
        todo!()
    }

    #[test]
    fn test_advance_fully_loaded_advance_partial() {
        todo!()
    }

    #[test]
    fn test_advance_partially_loaded_advance_partial() {
        todo!()
    }

    #[test]
    fn test_advance_partially_loaded_advance_all() {
        todo!()
    }
}
