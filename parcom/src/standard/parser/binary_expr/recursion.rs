use std::marker::PhantomData;

use crate::{ParseResult, Parser, RewindStream};

use super::{Associativity, Operator};

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

        loop {
            let anchor = rest.anchor();
            let (op, r) = match self.parser_op.parse(rest) {
                Ok((op, r)) if op.precedence() >= precedence => (op, r),
                Ok((_, r)) => {
                    rest = r.rewind(anchor);
                    break;
                }
                Err((_, r)) => {
                    rest = r.rewind(anchor);
                    break;
                }
            };

            let next_prec = match op.associativity() {
                Associativity::Left => op.precedence() + 1,
                Associativity::Right => op.precedence(),
            };

            let (rhs, r) = self.parse_impl(r, next_prec)?;

            lhs = op.construct(lhs, rhs);
            rest = r;
        }

        Ok((lhs, rest))
    }
}
