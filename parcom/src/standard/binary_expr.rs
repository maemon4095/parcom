pub enum Associativity {
    Left,
    Right,
}

pub trait Operator {
    type Expr;
    fn construct(self, lhs: Self::Expr, rhs: Self::Expr) -> Self::Expr;
    fn precedence(&self) -> usize;
    fn associativity(&self) -> Associativity;
}
