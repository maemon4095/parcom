use core::panic;
use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use crate::{RewindStream, Stream};

use super::{buffer_writer::BufferStaging, BufferWriter, StreamSource};
// asyncフレンドリーな形にしたい．Arcを使って上手く作れないか．
// async版と同期版を分けて作るか．
// async版はwriterを公開する形で作る．channelのようなapi

struct FromSource<T, S: StreamSource<T>> {
    source: RefCell<S>,
    offset: usize,
    is_completed: RefCell<bool>,
    segments: RefCell<Option<(Rc<Node<T>>, Rc<Node<T>>)>>,
    marker: PhantomData<T>,
}

impl<T, S: StreamSource<T>> FromSource<T, S> {
    pub fn new(source: S) -> Self {
        Self {
            source: RefCell::new(source),
            offset: 0,
            is_completed: RefCell::new(false),
            segments: RefCell::new(None),
            marker: PhantomData,
        }
    }

    fn append(&self, node: Rc<Node<T>>) -> (&Rc<Node<T>>, &Rc<Node<T>>) {
        match &mut *self.segments.borrow_mut() {
            Some((_, tail)) => {
                let old = std::mem::replace(&mut *tail.next.borrow_mut(), Some(node.clone()));
                assert!(old.is_none());
                *tail = node;
            }
            segments @ None => {
                *segments = Some((node.clone(), node));
            }
        }

        let (head, tail) = unsafe { self.segments.try_borrow_unguarded() }
            .unwrap()
            .as_ref()
            .unwrap();

        (head, tail)
    }

    fn read(&self) -> Option<Rc<Node<T>>> {
        if *self.is_completed.borrow() {
            return None;
        }
        let mut writer = Writer { vec: None };
        let res = self.source.borrow_mut().request(&mut writer);
        if res.is_completed {
            *self.is_completed.borrow_mut() = true;
        }

        writer.vec.map(|vec| {
            Rc::new(Node {
                vec,
                next: RefCell::new(None),
            })
        })
    }
}

struct Writer<T> {
    vec: Option<Vec<T>>,
}

impl<T> BufferWriter<T> for Writer<T> {
    fn stage(&mut self, min_capacity: usize) -> super::buffer_writer::BufferStaging<'_, T> {
        if self.vec.is_some() {
            panic!();
        }
        let mut vec = Vec::with_capacity(min_capacity);
        let ptr = vec.as_mut_ptr();
        let len = vec.len();
        self.vec = Some(vec);

        BufferStaging::new(ptr, len)
    }

    fn advance(&mut self, completion: super::buffer_writer::BufferCompletion<T>) {
        if self.vec.is_none() {
            panic!();
        }

        let vec = self.vec.as_mut().unwrap();
        if !vec.as_ptr().eq(&completion.ptr) {
            panic!();
        }
        if vec.len() < completion.len {
            panic!();
        }
        unsafe { vec.set_len(completion.len) }
    }
}

pub struct Anchor<T> {
    offset: usize,
    node: Option<Rc<Node<T>>>,
}

impl<T> Anchor<T> {
    fn empty() -> Self {
        Anchor {
            offset: 0,
            node: None,
        }
    }
}

pub struct Segments<'a, T, S: StreamSource<T>> {
    host: &'a FromSource<T, S>,
    offset: usize,
    current: Option<&'a Node<T>>,
}

impl<'a, T, S: StreamSource<T>> Iterator for Segments<'a, T, S> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            Some(node) => {
                let next = unsafe { node.next.try_borrow_unguarded() }
                    .unwrap()
                    .as_ref()
                    .map(|e| e.as_ref());

                self.current = next;

                let slice = &node.vec[self.offset..];
                self.offset = 0;
                Some(slice)
            }
            None => {
                let Some(next) = self.host.read() else {
                    return None;
                };
                let (_, node) = self.host.append(next);
                self.current = Some(node);

                let slice = &node.vec[self.offset..];
                self.offset = 0;

                Some(slice)
            }
        }
    }
}

struct Node<T> {
    vec: Vec<T>,
    next: RefCell<Option<Rc<Node<T>>>>,
}

impl<T, S: StreamSource<T>> Stream for FromSource<T, S> {
    type Item = T;
    type Iter<'a> = Segments<'a, T, S> where Self: 'a;

    fn segments(&self) -> Self::Iter<'_> {
        Segments {
            host: self,
            offset: self.offset,
            current: unsafe { self.segments.try_borrow_unguarded() }
                .unwrap()
                .as_ref()
                .map(|(_, t)| t.as_ref()),
        }
    }

    fn advance(mut self, count: usize) -> Self {
        let Some((mut head, mut tail)) = self.segments.replace(None) else {
            return self;
        };
        let mut rest = count;
        let mut offset = self.offset;

        loop {
            let len = head.vec.len() - offset;

            if rest <= len {
                offset = rest;
                break;
            }

            let next = match head.next.replace(None) {
                Some(n) => n,
                None => {
                    let n = match self.read() {
                        Some(n) => n,
                        None => break,
                    };
                    tail = n.clone();
                    n
                }
            };
            head = next;
            rest -= len;
            offset = 0;
        }

        self.offset = offset;
        self.segments = RefCell::new(Some((head, tail)));

        self
    }
}

impl<T, S: StreamSource<T>> RewindStream for FromSource<T, S> {
    type Anchor = Anchor<T>;

    fn anchor(&self) -> Self::Anchor {
        let reference = self.segments.borrow();
        let head = match &*reference {
            Some(t) => &t.0,
            None => {
                let Some(next) = self.read() else {
                    return Anchor::empty();
                };
                self.append(next).0
            }
        };

        Anchor {
            offset: self.offset,
            node: Some(head.clone()),
        }
    }

    fn rewind(mut self, anchor: Self::Anchor) -> Self {
        let segments = match anchor.node {
            Some(h) => {
                // stream must not be empty.
                let segments = self.segments.into_inner();
                assert!(segments.is_some());
                segments.map(|(_, t)| (h, t))
            }
            None => {
                // this stream must be empty
                let segments = self.segments.into_inner();
                assert!(segments.is_none());
                None
            }
        };

        self.offset = anchor.offset;
        self.segments = RefCell::new(segments);

        self
    }
}
