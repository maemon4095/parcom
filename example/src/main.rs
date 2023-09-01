use orthoparcom::foreign::parser;
use orthoparcom::standard::StandardExtension;
use orthoparcom::*;

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
