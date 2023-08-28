use crate::{Parser, Stream};

pub fn atom(str: &str) -> Atom<'_> {
    Atom { str }
}

pub struct Atom<'a> {
    str: &'a str,
}

impl<'a, S: Stream<Segment = str>> Parser<S> for Atom<'a> {
    type Output = &'a str;
    type Error = ();

    fn parse(&self, input: S) -> Result<(Self::Output, S), (Self::Error, S)> {
        let chars = self.str.chars();
        let target = input.segments().flat_map(|s| s.chars());
        if target.zip(chars).all(|(l, r)| l == r) {
            Ok((self.str, input.advance(self.str.len())))
        } else {
            Err(((), input))
        }
    }
}
