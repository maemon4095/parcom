use crate::ParseResult::{self, *};

use crate::{Parser, Stream};

pub fn atom(str: &str) -> Atom<'_> {
    Atom { str }
}

pub fn atom_char(char: char) -> AtomChar {
    AtomChar { char }
}

pub struct Atom<'a> {
    str: &'a str,
}

impl<'a, S: Stream<Segment = str>> Parser<S> for Atom<'a> {
    type Output = &'a str;
    type Error = ();

    fn parse(&self, input: S) -> ParseResult<S, Self::Output, Self::Error> {
        let mut chars = self.str.chars();
        let mut target = input.segments().flat_map(|s| s.chars());

        let mut consumed = 0;
        loop {
            let Some(c) = chars.next() else {
                drop(target);
                return Done(self.str, input.advance(consumed));
            };

            match target.next() {
                Some(t) if t == c => {
                    consumed += 1;
                }
                _ => {
                    drop(target);
                    return Fail((), input);
                }
            }
        }
    }
}

pub struct AtomChar {
    char: char,
}

impl<S: Stream<Segment = str>> Parser<S> for AtomChar {
    type Output = char;
    type Error = ();

    fn parse(&self, input: S) -> crate::ParseResult<S, Self::Output, Self::Error> {
        let head = input.segments().flat_map(|s| s.chars()).next();
        match head {
            Some(c) if c == self.char => Done(c, input.advance(1)),
            _ => Fail((), input),
        }
    }
}
