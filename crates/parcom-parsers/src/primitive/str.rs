use parcom_base::error::Miss;
use parcom_core::{ParseResult::*, Parser, ParserResult, SegmentIterator, Stream};
use std::ops::Deref;

pub fn atom(str: &str) -> Atom<'_> {
    Atom { str }
}

pub fn atom_char(char: char) -> AtomChar {
    AtomChar { char }
}

pub fn const_char<const C: char>() -> ConstChar<C> {
    ConstChar::<C>
}

pub fn any_char() -> AnyChar {
    AnyChar
}

pub struct Atom<'a> {
    str: &'a str,
}

impl<'a, S: Stream<Segment = str>> Parser<S> for Atom<'a> {
    type Output = &'a str;
    type Error = Miss<()>;

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let mut remain = self.str;
        let mut segment = input.segments();

        while let Some(segment) = segment.next(remain.len()).await {
            if segment.len() >= remain.len() {
                if segment.starts_with(remain) {
                    return Done(self.str, input.advance(self.str.len().into()).await);
                }
                break;
            }
            if !remain.starts_with(&*segment) {
                break;
            }

            remain = &remain[segment.len()..];
        }

        return Fail(().into(), input.into());
    }
}

pub struct AtomChar {
    char: char,
}

impl<S: Stream<Segment = str>> Parser<S> for AtomChar {
    type Output = char;
    type Error = Miss<()>;

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let mut segments = input.segments();

        let expected = self.char;
        loop {
            let Some(segment) = segments.next(expected.len_utf8()).await else {
                break;
            };

            if let Some(c) = segment.chars().next() {
                if c == expected {
                    return Done(c, input.advance(c.len_utf8().into()).await);
                }

                break;
            }
        }

        Fail(().into(), input.into())
    }
}

pub struct ConstChar<const C: char>;

impl<const C: char, S: Stream<Segment = str>> Parser<S> for ConstChar<C> {
    type Output = char;
    type Error = Miss<()>;

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let mut segments = input.segments();

        loop {
            let Some(segment) = segments.next(C.len_utf8()).await else {
                break;
            };
            let segment = segment.deref();

            if let Some(c) = segment.chars().next() {
                if c == C {
                    return Done(C, input.advance(C.len_utf8().into()).await);
                }

                break;
            }
        }

        Fail(().into(), input.into())
    }
}

pub struct AnyChar;

impl AnyChar {
    pub fn new() -> Self {
        Self
    }
}

impl<S: Stream<Segment = str>> Parser<S> for AnyChar {
    type Output = char;
    type Error = Miss<()>;

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let mut segments = input.segments();

        loop {
            let Some(segment) = segments.next(0).await else {
                break Fail(().into(), input.into());
            };
            let segment = segment.deref();

            if let Some(c) = segment.chars().next() {
                let rest = input.advance(c.len_utf8().into()).await;
                break Done(c, rest);
            }
        }
    }
}
