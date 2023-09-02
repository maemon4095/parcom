use parcom::foreign::parser;
use parcom::standard::StandardExtension;
use parcom::*;

fn main() {
    let atom = parser::str::atom("a");
    let rep = atom.repeat(..4);
    let source = foreign::stream::StrStream::new("aaaaaabbbccc");

    match rep.parse(source) {
        Ok((v, r)) => {
            println!("{:?} {:?}", v, r.location(0));
        }
        Err((e, r)) => {
            println!("{:?} {:?}", e, r.location(0));
        }
    }
}
