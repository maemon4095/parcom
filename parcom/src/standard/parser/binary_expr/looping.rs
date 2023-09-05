use std::marker::PhantomData;

use super::Operator;
use crate::{standard::binary_expr::Associativity, ParseResult, Parser, RewindStream};

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
        let (lhs, rest) = match self.parser_term.parse(input) {
            Ok(t) => t,
            Err((e, r)) => return Err((e, r)),
        };

        let anchor = rest.anchor();
        let (op, mut rest) = match self.parser_op.parse(rest) {
            Ok((op, r)) if op.precedence() >= precedence => (op, r),
            Ok((_, r)) => return Ok((lhs, r.rewind(anchor))),
            Err((_, r)) => return Ok((lhs, r.rewind(anchor))),
        };

        let mut stack = vec![(lhs, op)];

        let expr = loop {
            let prec = stack.last().map(|(_, op)| next_precedence(op)).unwrap();
            let (term, r) = match self.parser_term.parse(rest) {
                Ok(t) => t,
                Err(t) => return Err(t),
            };

            let anchor = r.anchor();
            match self.parser_op.parse(r) {
                Ok((op, r)) if op.precedence() >= prec => {
                    rest = r;
                    stack.push((term, op));
                }
                Ok((op, r)) => {
                    rest = r;
                    let (lhs, operator) = stack.pop().unwrap();
                    stack.push((operator.construct(lhs, term), op));
                    continue;
                }
                Err((_, r)) => {
                    rest = r.rewind(anchor);
                    let mut rhs = term;
                    while let Some((lhs, op)) = stack.pop() {
                        rhs = op.construct(lhs, rhs);
                    }
                    break rhs;
                }
            };
        };

        return Ok((expr, rest));

        fn next_precedence<T: Operator>(op: &T) -> usize {
            match op.associativity() {
                Associativity::Left => op.precedence() + 1,
                Associativity::Right => op.precedence(),
            }
        }
    }
}
