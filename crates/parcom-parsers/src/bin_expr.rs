mod parser;

pub use parser::BinExprParser;

pub enum Associativity {
    Left,
    Right,
}

pub trait Operator {
    fn precedence(&self) -> usize;
    fn associativity(&self) -> Associativity;
}
