use parcom::parsers::primitive::atom;
use parcom::prelude::*;

#[cfg_attr(test, test)]
pub fn main() {
    pollster::block_on(async {
        println!("----- repeat example -----\n");

        let parser = atom("text").map(|_| "text").repeat();
        let input = "texttexttextaaaa";

        match parser.parse(input).await {
            Ok((v, r)) => {
                println!("result: {:?}", v.0);
                println!("  rest: {}", r)
            }
            Err(_) => unreachable!(),
        }
    });
}
