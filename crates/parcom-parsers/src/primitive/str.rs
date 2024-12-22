use parcom_core::{Never, ParseResult::*, Parser, ParserResult, SegmentIterator, Stream};
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

pub struct Atom<'a> {
    str: &'a str,
}

impl<'a, S: Stream<Segment = str>> Parser<S> for Atom<'a> {
    type Output = &'a str;
    type Error = ();
    type Fault = Never;

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

        return Fail((), input.into());
    }
}

pub struct AtomChar {
    char: char,
}

impl<S: Stream<Segment = str>> Parser<S> for AtomChar {
    type Output = char;
    type Error = ();
    type Fault = Never;

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

        Fail((), input.into())
    }
}

pub struct ConstChar<const C: char>;

impl<const C: char, S: Stream<Segment = str>> Parser<S> for ConstChar<C> {
    type Output = char;
    type Error = ();
    type Fault = Never;

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

        Fail((), input.into())
    }
}
