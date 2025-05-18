use std::{
    cell::OnceCell, future::Future, mem::ManuallyDrop, ops::DerefMut, pin::Pin, sync::Arc,
    task::Poll,
};

use crate::stream_control::{vec_control::VecControl, Response};
use parcom_core::{SegmentIterator, Stream, StreamSegment};
use parcom_streams_core::StreamSource;

// 別スレッド版を作ってもいいかも。
#[derive(Debug, Clone)]
pub struct GenericStream<T: 'static + Default, S: StreamSource<Segment = [T]>> {
    source: S,
    offset: usize,
    pair: Option<(NodeSlot<T>, NodeSlot<T>)>,
}

impl<T: 'static + Default, S: StreamSource<Segment = [T]>> GenericStream<T, S> {
    pub const fn new(source: S) -> Self {
        Self {
            source,
            offset: 0,
            pair: None,
        }
    }
}

pub struct GenericStreamStrategy {
    min_capacity: usize,        // セグメントの最小容量
    unused_ratio_limit: usize, // セグメント内で利用されていない部分の割合の上限。利用されていない部分の割合がこの値を上回った場合、セグメントは縮小される。
    preallocation_limit: usize, // size_hintをもとに確保する容量の上限。
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

#[derive(Debug, Clone)]
struct Node<T: 'static + Default> {
    segment: Vec<T>, // UnsafeCellに変えて、後から追記できるようにする。Strategyを与えて、セグメントの最小容量とかを指定できるようにするか。
    next: OnceCell<NodeSlot<T>>,
}

pub struct Next<'a, T: 'static + Default, S: 'a + StreamSource<Segment = [T]>> {
    state: NextState<'a, T, S>,
}

enum NextState<'a, T: 'static + Default, S: 'a + StreamSource<Segment = [T]>> {
    Terminal,
    Loaded {
        node: &'a NodeSlot<T>,
        offset: usize,
    },
    Load {
        pair: &'a mut Option<(NodeSlot<T>, NodeSlot<T>)>,
        next: ManuallyDrop<S::Next<'a, VecControl<T, S::Error>>>,
    },
    Done,
}

impl<'a, T: 'static + Default, S: StreamSource<Segment = [T]>> Future for Next<'a, T, S> {
    type Output = Result<Option<&'a [T]>, S::Error>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        unsafe {
            match &mut self.get_unchecked_mut().state {
                me @ NextState::Loaded { .. } => {
                    let NextState::Loaded { node, offset } = me else {
                        unreachable!()
                    };
                    let offset = *offset;
                    let node = *node;
                    *me = NextState::Done;

                    let segment = match node {
                        NodeSlot::Some(node) => Some(&node.segment.as_slice()[offset..]),
                        NodeSlot::Terminal => None,
                    };
                    Poll::Ready(Ok(segment))
                }
                me @ NextState::Load { .. } => {
                    let NextState::Load { next, .. } = me else {
                        unreachable!()
                    };

                    let Poll::Ready(res) = Pin::new_unchecked(next.deref_mut()).poll(cx) else {
                        return Poll::Pending;
                    };

                    ManuallyDrop::drop(next);
                    let NextState::Load { pair, .. } = std::mem::replace(me, NextState::Done)
                    else {
                        unreachable!()
                    };

                    let node = match res {
                        Response::Cancel(_, e) => {
                            return Poll::Ready(Err(e));
                        }
                        Response::Advance(segment) => NodeSlot::Some(Arc::new(Node {
                            segment,
                            next: OnceCell::new(),
                        })),
                        Response::Finish(_) => NodeSlot::Terminal,
                    };

                    let segment: &'a Vec<T> = match pair {
                        Some((_, tail_slot)) => {
                            let NodeSlot::Some(tail) = tail_slot else {
                                unreachable!()
                            };

                            let r = tail.next.set(node.clone());
                            debug_assert!(
                                r.is_ok(),
                                "next of tail node must be uninitialized on load."
                            );

                            *tail_slot = node;

                            let NodeSlot::Some(tail) = tail_slot else {
                                return Poll::Ready(Ok(None));
                            };
                            &tail.segment
                        }
                        p @ None => {
                            let (_, tail) = p.get_or_insert((node.clone(), node));
                            let NodeSlot::Some(tail) = tail else {
                                unreachable!()
                            };

                            &tail.segment
                        }
                    };

                    *me = NextState::Done;

                    Poll::Ready(Ok(Some(segment)))
                }
                me @ NextState::Terminal => {
                    *me = NextState::Done;
                    Poll::Ready(Ok(None))
                }
                NextState::Done => panic!("`poll` must not be called on completed future."),
            }
        }
    }
}

impl<'a, T: 'static + Default, S: StreamSource<Segment = [T]>> Drop for Next<'a, T, S> {
    fn drop(&mut self) {
        match &mut self.state {
            NextState::Load { next, .. } => unsafe { ManuallyDrop::drop(next) },
            _ => {}
        }
    }
}

pub struct Iter<'a, T: 'static + Default, S: 'a + StreamSource<Segment = [T]>> {
    source: &'a mut S,
    pair: &'a mut Option<(NodeSlot<T>, NodeSlot<T>)>,
    offset: usize,
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
                            Next {
                                state: NextState::Loaded { node, offset: 0 },
                            }
                        }
                        None => {
                            self.state = IterState::Loading;
                            let control = VecControl::new(Vec::with_capacity(size_hint));
                            Next {
                                state: NextState::Load {
                                    pair: &mut *self.pair,
                                    next: ManuallyDrop::new(self.source.next(control, size_hint)),
                                },
                            }
                        }
                    }
                }
                NodeSlot::Terminal => {
                    let IterState::Iterating(_) = &self.state else {
                        unreachable!()
                    };
                    Next {
                        state: NextState::Terminal,
                    }
                }
            },
            IterState::Initial => match &self.pair {
                Some((head, _)) => {
                    self.state = IterState::Iterating(head.clone());
                    let IterState::Iterating(node) = &self.state else {
                        unreachable!()
                    };
                    Next {
                        state: NextState::Loaded {
                            node,
                            offset: self.offset,
                        },
                    }
                }
                None => {
                    self.state = IterState::Loading;
                    let control = VecControl::new(Vec::with_capacity(size_hint));
                    Next {
                        state: NextState::Load {
                            pair: &mut *self.pair,
                            next: ManuallyDrop::new(self.source.next(control, size_hint)),
                        },
                    }
                }
            },
            IterState::Loading => {
                if let Some((_, NodeSlot::Terminal)) = &self.pair {
                    return Next {
                        state: NextState::Terminal,
                    };
                };
                let control = VecControl::new(Vec::with_capacity(size_hint));
                Next {
                    state: NextState::Load {
                        pair: &mut *self.pair,
                        next: ManuallyDrop::new(self.source.next(control, size_hint)),
                    },
                }
            }
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
            source: &mut self.source,
            pair: &mut self.pair,
            offset: self.offset,
            state: IterState::Initial,
        }
    }

    fn advance(self, delta: <Self::Segment as StreamSegment>::Length) -> Self::Advance {
        let source = self.source;

        let mut remain = delta;
        let fut = Box::pin(async move {
            let (head, tail) = match self.pair {
                Some((h, t)) => (h, t),
                None => {
                    return advance_load(source, delta).await;
                }
            };

            let mut current = Some(head);
            loop {
                let Some(slot) = current else {
                    return advance_load(source, remain).await;
                };

                let NodeSlot::Some(node) = slot else {
                    return Ok(GenericStream {
                        source,
                        offset: 0,
                        pair: Some((NodeSlot::Terminal, NodeSlot::Terminal)),
                    });
                };

                let len = node.segment.len();
                if len > remain {
                    return Ok(GenericStream {
                        source,
                        offset: remain,
                        pair: Some((NodeSlot::Some(node), tail)),
                    });
                }

                remain -= len;
                current = node.next.get().cloned();
            }
        });

        Advance { fut }
    }
}

async fn advance_load<T: 'static + Default, S: 'static + StreamSource<Segment = [T]>>(
    mut source: S,
    mut remain: usize,
) -> Result<GenericStream<T, S>, S::Error> {
    loop {
        let control = VecControl::new(Vec::new());
        let res = source.next(control, remain).await;

        match res {
            Response::Advance(segment) => {
                if segment.len() > remain {
                    let node = Arc::new(Node {
                        segment,
                        next: OnceCell::new(),
                    });

                    return Ok(GenericStream {
                        source,
                        offset: remain,
                        pair: Some((NodeSlot::Some(node.clone()), NodeSlot::Some(node))),
                    });
                } else {
                    remain -= segment.len();
                }
            }
            Response::Cancel(_, e) => return Err(e),
            Response::Finish(segment) => {
                if segment.len() > remain {
                    let node = Arc::new(Node {
                        segment,
                        next: OnceCell::new(),
                    });

                    return Ok(GenericStream {
                        source,
                        offset: remain,
                        pair: Some((NodeSlot::Some(node.clone()), NodeSlot::Some(node))),
                    });
                } else {
                    return Ok(GenericStream {
                        source,
                        offset: 0,
                        pair: Some((NodeSlot::Terminal, NodeSlot::Terminal)),
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::stream_source::iterator_source::IteratorSource;

    use super::*;

    #[test]
    fn test_load_all() {
        let array: &[&[usize]] = &[&[0], &[1, 2, 3], &[], &[4, 5, 6]];
        let source = IteratorSource::new(array);
        let mut stream = GenericStream::new(source);

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
        test_load_all();
        let array: &[&[usize]] = &[&[0], &[1, 2, 3], &[], &[4, 5, 6]];
        let source = IteratorSource::new(array);
        let mut stream = GenericStream::new(source);

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
        todo!()
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
