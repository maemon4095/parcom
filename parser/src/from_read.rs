use std::{io::Read, rc::Rc};

use crate::ParseStream;

pub struct FromRead<T: Read> {
    source: T,

    offset: usize,
    segments: Option<(Rc<Node>, Rc<Node>)>,
}

pub struct Anchor {
    offset: usize,
    node: Rc<Node>,
}

pub struct Segments<'a> {
    offset: usize,
    current: Option<&'a Node>,
}

impl<'a> Iterator for Segments<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let Some(segment) = std::mem::replace(&mut self.current, None) else {
            return None;
        };

        let item = &segment.buf[self.offset..segment.len];

        self.offset = 0;
        self.current = segment.next.as_ref().map(|e| e.as_ref());

        Some(item)
    }
}

struct Node {
    buf: Box<[u8]>,
    len: usize,
    next: Option<Rc<Node>>,
}

impl<T: Read> ParseStream<u8> for FromRead<T> {
    type Location = usize;
    type Anchor = Anchor;
    type Segments<'a> = Segments<'a> where Self: 'a;

    fn segments(&self) -> Self::Segments<'_> {
        Segments {
            offset: self.offset,
            current: self.segments.as_ref().map(|e| e.0.as_ref()),
        }
    }

    fn location(&self, indes: usize) -> Self::Location {
        todo!()
    }

    fn anchor(&self) -> Self::Anchor {
        todo!()
    }

    fn advance(self, count: usize) -> Self {
        todo!()
    }

    fn rewind(self, anchor: Self::Anchor) -> Self {
        todo!()
    }
}
