use std::{cell::RefCell, io::Read, rc::Rc};

use crate::{RewindStream, Stream};
// TODO: bufferwriterを使ったより一般の実装に差し替える．
pub struct FromRead<T: Read> {
    source: RefCell<T>,
    offset: usize,
    is_completed: RefCell<bool>,
    segments: RefCell<Option<(Rc<Node>, Rc<Node>)>>,
}

impl<T: Read> FromRead<T> {
    pub fn new(source: T) -> Self {
        Self {
            source: RefCell::new(source),
            offset: 0,
            is_completed: RefCell::new(false),
            segments: RefCell::new(None),
        }
    }

    fn append(&self, node: Rc<Node>) -> (&Rc<Node>, &Rc<Node>) {
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

    fn read(&self) -> Option<Rc<Node>> {
        if *self.is_completed.borrow() {
            return None;
        }

        let mut buf = [0; 1024];

        let written = self.source.borrow_mut().read(&mut buf).unwrap();

        if written == 0 {
            *self.is_completed.borrow_mut() = true;
            return None;
        }

        let node = Rc::new(Node {
            vec: Vec::from(&buf[..written]),
            next: RefCell::new(None),
        });

        Some(node)
    }
}

pub struct Anchor {
    offset: usize,
    node: Option<Rc<Node>>,
}

impl Anchor {
    fn empty() -> Self {
        Anchor {
            offset: 0,
            node: None,
        }
    }
}

pub struct Segments<'a, T: Read> {
    host: &'a FromRead<T>,
    offset: usize,
    current: Option<&'a Node>,
}

impl<'a, T: Read> Iterator for Segments<'a, T> {
    type Item = &'a [u8];

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

struct Node {
    vec: Vec<u8>,
    next: RefCell<Option<Rc<Node>>>,
}

impl<T: Read> Stream for FromRead<T> {
    type Segment = [u8];
    type Iter<'a> = Segments<'a, T> where Self: 'a;

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

impl<T: Read> RewindStream for FromRead<T> {
    type Anchor = Anchor;

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
