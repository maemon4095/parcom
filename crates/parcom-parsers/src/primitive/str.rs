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

    fn parse(&self, input: S) -> ParserResult<S, Self> {
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
                    return Fail((), input.into());
                }
            }
        }
    }
}

pub struct AtomChar {
    char: char,
}

impl<S: ParcomStream<Segment = str>> Parser<S> for AtomChar {
    type Output = char;
    type Error = ();
    type Fault = Never;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
        let head = input.segments().flat_map(|s| s.chars()).next();
        match head {
            Some(c) if c == self.char => Done(c, input.advance(1)),
            _ => Fail((), input.into()),
        }
    }
}

pub struct ConstChar<const C: char>;

impl<const C: char, S: ParcomStream<Segment = str>> Parser<S> for ConstChar<C> {
    type Output = char;
    type Error = ();
    type Fault = Never;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
        let head = input.segments().flat_map(|s| s.chars()).next();
        match head {
            Some(c) if c == C => Done(C, input.advance(1)),
            _ => Fail((), input.into()),
        }
    }
}
