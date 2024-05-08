use parcom::parsers::primitive::str::atom;
use parcom::prelude::*;

#[cfg_attr(test, test)]
pub fn main() {
    println!("----- repeat example -----\n");

    let parser = atom("text").repeat(..);
    let input = "texttexttextaaaa";

    match parser.parse(input) {
        Done(v, r) => {
            println!("result: {:?}", v.0);
            println!("  rest: {}", r)
        }
        Fail(e, _) => e.never(),
        Fatal(e, _) => e.never(),
    }
}
