use parser::standard::StandardExtension;
use parser::*;

fn main() {
    let atom = Atom { str: "a" };

    let rep = atom
        .repeat(..=4)
        .join(|input: foreign::stream::StrStream<'static>| {
            if input.segments().flat_map(|s| s.chars()).eq(['b', 'b', 'b']) {
                Ok(("bbb", input.advance(3)))
            } else {
                Err(((), input))
            }
        });
    let source = foreign::stream::StrStream::new("aaaabbbccc");

    match rep.parse(source) {
        Ok((v, r)) => {
            println!("{:?} {:?}", v, r.location(0));
        }
        Err((e, r)) => {
            println!("{:?} {:?}", e, r.location(0));
        }
    }
}

struct Atom {
    str: &'static str,
}

impl<S: ParseStream<Segment = str>> Parser<S> for Atom {
    type Output = &'static str;
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
