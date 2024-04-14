#![cfg_attr(test, cfg(test))]
mod binary_expr;

#[cfg_attr(test, test)]
pub fn main() {
    binary_expr::main();
}
