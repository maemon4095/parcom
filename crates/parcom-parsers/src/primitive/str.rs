use futures::StreamExt;
use parcom_core::{Never, ParcomStream, ParseResult::*, Parser, ParserResult};

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

impl<'a, S: ParcomStream<Segment = str>> Parser<S> for Atom<'a> {
    type Output = &'a str;
    type Error = ();
    type Fault = Never;

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let mut remain = self.str;
        let mut nodes = input.nodes();

        while let Some(node) = nodes.next().await {
            let segment = node.as_ref();

            if !segment.starts_with(remain) {
                break;
            }

            if segment.len() >= remain.len() {
                return Done(self.str, input.advance(self.str.len()).await);
            }

            remain = &remain[segment.len()..];
        }

        return Fail((), input.into());
    }
}

pub struct AtomChar {
    char: char,
}

impl<S: ParcomStream<Segment = str>> Parser<S> for AtomChar {
    type Output = char;
    type Error = ();
    type Fault = Never;

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let mut nodes = input.nodes();

        loop {
            let Some(node) = nodes.next().await else {
                break;
            };

            let segment = node.as_ref();

            if let Some(c) = segment.chars().next() {
                if c == self.char {
                    return Done(self.char, input.advance(1).await);
                }

                break;
            }
        }

        Fail((), input.into())
    }
}

pub struct ConstChar<const C: char>;

impl<const C: char, S: ParcomStream<Segment = str>> Parser<S> for ConstChar<C> {
    type Output = char;
    type Error = ();
    type Fault = Never;

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let mut nodes = input.nodes();

        loop {
            let Some(node) = nodes.next().await else {
                break;
            };

            let segment = node.as_ref();

            if let Some(c) = segment.chars().next() {
                if c == C {
                    return Done(C, input.advance(1).await);
                }

                break;
            }
        }

        Fail((), input.into())
    }
}
