#![cfg_attr(test, cfg(test))]
mod binary_expr;
mod cache;
mod line_chart;

#[cfg_attr(test, test)]
fn main() {
    binary_expr::main();
    cache::main();
}
