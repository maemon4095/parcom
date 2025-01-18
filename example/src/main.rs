#![cfg_attr(test, cfg(test))]

mod bin_expr;
mod cache;
mod repeat;
mod utils;

#[cfg_attr(test, test)]
fn main() {
    bin_expr::main();
    cache::main();
    repeat::main();
}
