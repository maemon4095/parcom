#![cfg_attr(test, cfg(test))]

mod line_chart;
mod parse;
mod parser;

#[cfg_attr(test, test)]
fn main() {
    parse::main();
    parser::main();
}
