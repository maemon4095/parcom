use std::marker::PhantomData;

use crate::{ParseResult, Parser, RewindStream};
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

/// expr = expr op expr / term
pub struct BinaryExprParser<S: RewindStream, PTerm: Parser<S>, POp: Parser<S>>
where
    POp::Output: Operator,
{
    parser_term: PTerm,
    parser_op: POp,
    marker: PhantomData<S>,
}

impl<S, E, PTerm, POp> Parser<S> for BinaryExprParser<S, PTerm, POp>
where
    S: RewindStream,
    POp::Output: Operator<Expr = E>,
    PTerm: Parser<S, Output = E>,
    POp: Parser<S>,
{
    type Output = E;
    type Error = PTerm::Error;

    fn parse(&self, input: S) -> ParseResult<S, Self::Output, Self::Error> {
        self.parse_impl(input, 0)
    }
}

impl<S, E, PTerm, POp> BinaryExprParser<S, PTerm, POp>
where
    S: RewindStream,
    POp::Output: Operator<Expr = E>,
    PTerm: Parser<S, Output = E>,
    POp: Parser<S>,
{
    pub fn new(parser_term: PTerm, parser_op: POp) -> Self {
        Self {
            parser_term,
            parser_op,
            marker: PhantomData,
        }
    }

    fn parse_impl(&self, input: S, precedence: usize) -> ParseResult<S, E, PTerm::Error> {
        let (mut lhs, mut rest) = match self.parser_term.parse(input) {
            Ok(t) => t,
            Err((e, r)) => return Err((e, r)),
        };

        let mut last_anchor = rest.anchor();

        loop {
            let (op, r) = match self.parser_op.parse(rest) {
                Ok((op, r)) if op.precedence() >= precedence => (op, r),
                Ok((_, r)) => {
                    rest = r.rewind(last_anchor);
                    break;
                }
                Err((_, r)) => {
                    rest = r.rewind(last_anchor);
                    break;
                }
            };

            let next_prec = match op.associativity() {
                Associativity::Left => precedence + 1,
                Associativity::Right => precedence,
            };

            let (rhs, r) = self.parse_impl(r, next_prec)?;

            lhs = op.construct(lhs, rhs);
            last_anchor = r.anchor();
            rest = r;
        }

        Ok((lhs, rest))
    }
}
